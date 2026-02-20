class Hindsight < Formula
  desc "20/20 hindsight for your Claude Code sessions"
  homepage "https://github.com/Codestz/claude-hindsight"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Codestz/claude-hindsight/releases/download/v#{version}/hindsight-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_AARCH64_MACOS_SHA256"
    else
      url "https://github.com/Codestz/claude-hindsight/releases/download/v#{version}/hindsight-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_X86_64_MACOS_SHA256"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/Codestz/claude-hindsight/releases/download/v#{version}/hindsight-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_AARCH64_LINUX_SHA256"
    else
      url "https://github.com/Codestz/claude-hindsight/releases/download/v#{version}/hindsight-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_X86_64_LINUX_SHA256"
    end
  end

  def install
    bin.install "hindsight"
  end

  test do
    system "#{bin}/hindsight", "--version"
  end
end
