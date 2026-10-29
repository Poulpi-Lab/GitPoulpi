# TEMPLATE, NOT PUBLISHED. see the release-and-packaging design ("Homebrew").
#
# Formula for a tap repository (Formula/gitpoulpi.rb of a `homebrew-gitpoulpi` repo, installed with
# `brew install <owner>/gitpoulpi/gitpoulpi`). It installs the official release archives, no compilation.
# Replace @VERSION@ and the @SHA256_...@ values (first field of each archive's .sha256), then
# `brew install --build-from-source ./gitpoulpi.rb` and `brew test gitpoulpi` (macOS is untested so far).
#
# A formula download is not quarantined, so Gatekeeper does not block the unsigned binaries. The
# `.gitpoulpi-dist` marker is not installed: GitPoulpi never replaces its own files, brew upgrades it.
class Gitpoulpi < Formula
  desc "Git client for the terminal with a clickable commit graph, a CLI and an MCP server"
  homepage "https://github.com/Poulpi-Lab/GitPoulpi"
  version "@VERSION@"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Poulpi-Lab/GitPoulpi/releases/download/releases/v#{version}/gitpoulpi-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "@SHA256_MACOS_ARM64@"
    end
    on_intel do
      url "https://github.com/Poulpi-Lab/GitPoulpi/releases/download/releases/v#{version}/gitpoulpi-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "@SHA256_MACOS_X86_64@"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Poulpi-Lab/GitPoulpi/releases/download/releases/v#{version}/gitpoulpi-#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "@SHA256_LINUX_X86_64@"
    end
  end

  def install
    bin.install "gitpoulpi", "gitpoulpi-tui", "gitpoulpi-mcp", "gitpoulpi-launch"
    doc.install "README.md", "CHANGELOG.md", "INSTALL.txt"
    prefix.install "LICENSE", "THIRD-PARTY-LICENSES.txt"
  end

  def caveats
    <<~EOS
      GitPoulpi needs a system git on the PATH (the Xcode command line tools or `brew install git`).
      `gitpoulpi` with no subcommand opens the terminal UI.
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/gitpoulpi --version")
  end
end
