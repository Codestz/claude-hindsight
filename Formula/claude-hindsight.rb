class ClaudeHindsight < Formula
  desc "20/20 hindsight for your Claude Code sessions"
  homepage "https://github.com/Codestz/claude-hindsight"
  version "1.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Codestz/claude-hindsight/releases/download/v#{version}/claude-hindsight-aarch64-apple-darwin.tar.gz"
      sha256 "08a8fcf14492c5e5be95b097825e3435457993ea90712c9daded9404d6691d5b"
    else
      url "https://github.com/Codestz/claude-hindsight/releases/download/v#{version}/claude-hindsight-x86_64-apple-darwin.tar.gz"
      sha256 "8dfa8d6eeeab2503bdac4402551f93c17982809babdf4e52cb7d95a2251c2303"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/Codestz/claude-hindsight/releases/download/v#{version}/claude-hindsight-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "023d5aee10e32c2777ad114ecec0de0a7e7ab815055dc23f67215c414b120328"
    else
      url "https://github.com/Codestz/claude-hindsight/releases/download/v#{version}/claude-hindsight-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "fd515cc39a9ab2ff8ec9497c44ffd2944830af95dc0ae223d11e17505e274fce"
    end
  end

  def install
    bin.install "claude-hindsight"
  end

  test do
    system "#{bin}/claude-hindsight", "--version"
  end
end
