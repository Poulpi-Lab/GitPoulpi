//! `general.keep_submodules_up_to_date`: after a checkout or pull that
//! changed the superproject, bring its submodules to the commits it now
//! records (`submodule update --init --recursive`). Only submodules that
//! are not already where the superproject records them are touched, so a
//! repository without submodules (the common case) costs one cheap read.

use gitpoulpi_core::domain::{
    CheckoutTarget, OperationKind, SubmoduleState, SubmoduleUpdateOptions,
};
use gitpoulpi_core::GitBackend;
use gitpoulpi_ipc::Request;

use crate::dispatch::{err_kind, Dispatched};

fn applies(request: &Request) -> bool {
    match request.command {
        OperationKind::Pull => true,
        OperationKind::Checkout => request
            .args
            .get("target")
            .and_then(|t| serde_json::from_value::<CheckoutTarget>(t.clone()).ok())
            .is_some_and(|t| {
                !matches!(
                    t,
                    CheckoutTarget::File { .. } | CheckoutTarget::Files { .. }
                )
            }),
        _ => false,
    }
}

/// Called with the finished result of `request`. Leaves `dispatched` alone
/// unless the operation succeeded, the setting is on and some submodule is
/// out of step - then updates them, and turns a failure into an error that
/// says the operation itself worked.
pub fn keep_up_to_date(backend: &dyn GitBackend, request: &Request, dispatched: &mut Dispatched) {
    if !cfg!(feature = "submodule") || !applies(request) || dispatched.response.result.is_err() {
        return;
    }
    let on = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().general.keep_submodules_up_to_date)
        .unwrap_or(false);
    if !on {
        return;
    }
    let Ok(repo) = crate::dispatch::repo_ref(request) else {
        return;
    };
    let out_of_step = backend
        .submodules(&repo)
        .map(|list| list.iter().any(|s| s.state != SubmoduleState::UpToDate))
        .unwrap_or(false);
    if !out_of_step {
        return;
    }
    crate::credentials::apply_from_config();
    let updated = backend.submodule_update(
        &repo,
        &SubmoduleUpdateOptions {
            paths: Vec::new(),
            init: true,
            recursive: true,
        },
    );
    dispatched.mutated = true;
    if let Err(e) = updated {
        let journal = dispatched.journal.take();
        *dispatched = err_kind(
            request.id,
            format!("the operation succeeded, but updating the submodules failed: {e}"),
            false,
            None,
        );
        dispatched.mutated = true;
        dispatched.journal = journal;
    }
}
