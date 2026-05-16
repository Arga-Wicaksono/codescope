# CodeScope (cs) — Homebrew Formula
# Install: brew tap arga-wicaksono/codescope
#         brew install codescope

class Codescope < Formula
  desc "Repository Intelligence Engine for AI & Developers"
  homepage "https://github.com/Arga-Wicaksono/codescope"
  version "1.3.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Arga-Wicaksono/codescope/releases/download/v#{version}/cs-aarch64-macos.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
    on_intel do
      url "https://github.com/Arga-Wicaksono/codescope/releases/download/v#{version}/cs-x86_64-macos.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/Arga-Wicaksono/codescope/releases/download/v#{version}/cs-aarch64-linux-musl.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
    on_intel do
      url "https://github.com/Arga-Wicaksono/codescope/releases/download/v#{version}/cs-x86_64-linux-musl.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
  end

  def install
    bin.install "cs"
    # Generate shell completions
    generate_completions_from_executable(bin/"cs", "completions", shells: [:bash, :zsh, :fish])
  end

  test do
    assert_match "CodeScope", shell_output("#{bin}/cs --help")
    assert_match "code", shell_output("#{bin}/cs explain '\\w+'")
  end
end
