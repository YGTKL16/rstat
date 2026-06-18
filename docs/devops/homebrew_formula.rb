# Homebrew Formula template for rstat-cli
# Place this in your homebrew-rstat tap repository as Formula/rstat-cli.rb
# This template is automatically updated by cargo-dist if tap is configured.

class RstatCli < Formula
  desc "Cross-platform statistics CLI — scipy.stats for the terminal"
  homepage "https://rstat.dev"
  version "0.1.0"
  license "BUSL-1.1"

  on_macos do
    on_arm do
      url "https://github.com/ygtkula/rstat/releases/download/v#{version}/rstat-cli-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    end
    on_intel do
      url "https://github.com/ygtkula/rstat/releases/download/v#{version}/rstat-cli-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/ygtkula/rstat/releases/download/v#{version}/rstat-cli-v#{version}-aarch64-unknown-linux-musl.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    end
    on_intel do
      url "https://github.com/ygtkula/rstat/releases/download/v#{version}/rstat-cli-v#{version}-x86_64-unknown-linux-musl.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    end
  end

  def install
    bin.install "rstat-cli" => "rstat"
  end

  test do
    assert_match "rstat-cli", shell_output("#{bin}/rstat --version")
  end
end
