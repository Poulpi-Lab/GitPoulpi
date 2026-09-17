//! History-rewrite recipes on disk (the history-rewrite design,
//! "Where scripts and recipes live"), the shape of the Workflow page's
//! shared / only-for-me pair:
//!
//! - **Shared with the repository:** `<repo>/.gitpoulpi/rewrite/<name>.toml`,
//!   a folder the user commits (the state folder's `.gitignore` is told to let
//!   it through).
//! - **Only for me:** `<config dir>/rewrite/<name>.toml`, available in every
//!   repository.
//!
//! A saved recipe never holds a secret value: [`Recipe::saved`] clears the
//! literals and a secret rule has to be given them again at each run.

use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::rewrite::Recipe;

use crate::repo_state::ensure_state_gitignore;
use crate::{Error, Result};

/// The folder name under the state folder and under the config dir.
pub const RECIPES_DIR: &str = "rewrite";

/// Where a recipe is kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipePlace {
    /// Committed with the repository, seen by the team.
    Shared,
    /// Under the user's config dir, seen by this user in every repository.
    Mine,
}

/// The folder of the shared recipes of the repository at `repo_root`.
pub fn shared_recipes_dir(repo_root: &Path) -> PathBuf {
    repo_root
        .join(gitpoulpi_config::LEGACY_DIR_NAME)
        .join(RECIPES_DIR)
}

/// The folder of this user's recipes.
pub fn my_recipes_dir(config_dir: &Path) -> PathBuf {
    config_dir.join(RECIPES_DIR)
}

/// A file name stem for a recipe name: lower-case letters, digits and `-`;
/// `recipe` for a name with none.
pub fn recipe_slug(name: &str) -> String {
    if name.chars().any(|c| c.is_ascii_alphanumeric()) {
        crate::slugify(name)
    } else {
        "recipe".to_string()
    }
}

/// The file `name` is saved to in `dir`.
pub fn recipe_path(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{}.toml", recipe_slug(name)))
}

/// One recipe file found by [`list_recipes`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeEntry {
    /// The recipe's own name, else the file stem.
    pub name: String,
    pub path: PathBuf,
}

/// Reads one recipe file.
pub fn load_recipe(path: &Path) -> Result<Recipe> {
    Ok(toml::from_str(&std::fs::read_to_string(path)?)?)
}

/// Writes `recipe` (secrets removed) to `<dir>/<slug of its name>.toml` and
/// returns the path. A shared save also lets the folder through the state
/// folder's ignore file.
pub fn save_recipe(dir: &Path, place: RecipePlace, recipe: &Recipe) -> Result<PathBuf> {
    let text = toml::to_string_pretty(&recipe.saved())?;
    std::fs::create_dir_all(dir)?;
    if place == RecipePlace::Shared {
        if let Some(state) = dir.parent() {
            ensure_state_gitignore(state)?;
            let_recipes_through(state)?;
        }
    }
    let path = recipe_path(dir, &recipe.name);
    std::fs::write(&path, text)?;
    Ok(path)
}

/// Appends the exception for the recipes folder to an ignore file GitPoulpi
/// wrote (`*` plus the exceptions); one the user wrote is left alone.
fn let_recipes_through(state: &Path) -> Result<()> {
    let ignore = state.join(".gitignore");
    let Ok(current) = std::fs::read_to_string(&ignore) else {
        return Ok(());
    };
    let ours = current.starts_with("# Written by GitPoulpi");
    if ours && !current.lines().any(|l| l.trim() == "!rewrite/") {
        let mut next = current;
        if !next.ends_with('\n') {
            next.push('\n');
        }
        next.push_str("!rewrite/\n!rewrite/*.toml\n");
        std::fs::write(&ignore, next)?;
    }
    Ok(())
}

/// The recipes in `dir`, by name; an unreadable file is skipped.
pub fn list_recipes(dir: &Path) -> Vec<RecipeEntry> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<RecipeEntry> = read
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .filter_map(|path| {
            let recipe = load_recipe(&path).ok()?;
            let stem = path.file_stem()?.to_string_lossy().into_owned();
            let name = if recipe.name.trim().is_empty() {
                stem
            } else {
                recipe.name
            };
            Some(RecipeEntry { name, path })
        })
        .collect();
    found.sort_by_key(|e| e.name.to_lowercase());
    found
}

/// Removes a saved recipe file.
pub fn delete_recipe(path: &Path) -> Result<()> {
    std::fs::remove_file(path).map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::rewrite::{
        ContentFind, ContentReplace, IdentityMatch, IdentityRule, IdentitySet, Rule, RuleKind,
        Scope, SecretText, Who,
    };

    fn recipe(name: &str) -> Recipe {
        Recipe::new(
            name,
            Scope::AllLocal,
            vec![
                Rule::new(RuleKind::Identity(IdentityRule {
                    when: IdentityMatch::email("old@example.com"),
                    set: IdentitySet {
                        name: Some("Alex Doe".into()),
                        email: Some("alex@example.org".into()),
                    },
                    apply_to: Who::Both,
                    include_taggers: true,
                    map_trailers: false,
                })),
                Rule::new(RuleKind::DateShift {
                    by_seconds: 7200,
                    apply_to: Default::default(),
                }),
                Rule::new(RuleKind::ContentReplace(ContentReplace {
                    paths: vec!["*.env".into()],
                    find: ContentFind::Secrets {
                        literals: vec![SecretText("hunter2".into())],
                        scan: true,
                    },
                    with: String::new(),
                    max_blob_bytes: None,
                })),
            ],
        )
    }

    #[test]
    fn a_recipe_round_trips_through_its_file_without_its_secret() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = my_recipes_dir(tmp.path());
        let original = recipe("Fix work email");
        let path = save_recipe(&dir, RecipePlace::Mine, &original).unwrap();
        assert_eq!(path, dir.join("fix-work-email.toml"));
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(!text.contains("hunter2"));
        let back = load_recipe(&path).unwrap();
        assert_eq!(back, original.saved());
        assert!(!back.holds_secrets());
    }

    #[test]
    fn shared_and_mine_are_separate_folders_and_listing_sorts_by_name() {
        let tmp = tempfile::tempdir().unwrap();
        let shared = shared_recipes_dir(tmp.path());
        let mine = my_recipes_dir(&tmp.path().join("cfg"));
        save_recipe(&shared, RecipePlace::Shared, &recipe("Zeta")).unwrap();
        save_recipe(&shared, RecipePlace::Shared, &recipe("alpha")).unwrap();
        save_recipe(&mine, RecipePlace::Mine, &recipe("Mine only")).unwrap();
        let names: Vec<_> = list_recipes(&shared).into_iter().map(|e| e.name).collect();
        assert_eq!(names, ["alpha", "Zeta"]);
        let mine_names: Vec<_> = list_recipes(&mine).into_iter().map(|e| e.name).collect();
        assert_eq!(mine_names, ["Mine only"]);
        assert!(shared.starts_with(tmp.path().join(".gitpoulpi")));
    }

    #[test]
    fn a_shared_save_lets_the_folder_through_the_state_ignore_file() {
        let tmp = tempfile::tempdir().unwrap();
        let shared = shared_recipes_dir(tmp.path());
        save_recipe(&shared, RecipePlace::Shared, &recipe("one")).unwrap();
        let ignore = std::fs::read_to_string(tmp.path().join(".gitpoulpi/.gitignore")).unwrap();
        assert!(ignore.lines().any(|l| l == "!rewrite/"));
        save_recipe(&shared, RecipePlace::Shared, &recipe("two")).unwrap();
        let again = std::fs::read_to_string(tmp.path().join(".gitpoulpi/.gitignore")).unwrap();
        assert_eq!(again.matches("!rewrite/\n").count(), 1);
    }

    #[test]
    fn an_unreadable_file_is_skipped_and_a_blank_name_gets_a_file_name() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = my_recipes_dir(tmp.path());
        save_recipe(&dir, RecipePlace::Mine, &recipe("")).unwrap();
        std::fs::write(dir.join("broken.toml"), "not = [toml").unwrap();
        let listed = list_recipes(&dir);
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].path, dir.join("recipe.toml"));
        assert_eq!(listed[0].name, "recipe");
    }
}
