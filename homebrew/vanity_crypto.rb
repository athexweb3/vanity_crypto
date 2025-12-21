class VanityCrypto < Formula
  desc "High-performance, secure, and beautiful Ethereum vanity address generator"
  homepage "https://github.com/athexweb3/vanity_crypto"
  version "0.1.0-alpha"
  
  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v#{version}/vc-macos-arm64"
    sha256 "REPLACE_WITH_ACTUAL_SHA256_AFTER_RELEASE_BUILD_MACOS" # MAC_SHA
  elsif OS.linux?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v#{version}/vc-linux-amd64"
    sha256 "REPLACE_WITH_ACTUAL_SHA256_AFTER_RELEASE_BUILD_LINUX" # LINUX_SHA
  end

  def install
    bin.install "vc-macos-arm64" => "vc" if OS.mac?
    bin.install "vc-linux-amd64" => "vc" if OS.linux?
  end

  test do
    assert_match "Usage:", shell_output("#{bin}/vc --help")
  end
end
