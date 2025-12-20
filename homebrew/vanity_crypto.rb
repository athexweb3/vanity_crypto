class VanityCrypto < Formula
  desc "High-performance, secure, and beautiful Ethereum vanity address generator"
  homepage "https://github.com/athexweb3/vanity_crypto"
  version "0.1.0"
  
  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v0.1.0/vanity_crypto-macos-arm64"
    sha256 "REPLACE_WITH_ACTUAL_SHA256_AFTER_RELEASE_BUILD_MACOS"
  elsif OS.linux?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v0.1.0/vanity_crypto-linux-amd64"
    sha256 "REPLACE_WITH_ACTUAL_SHA256_AFTER_RELEASE_BUILD_LINUX"
  end

  def install
    bin.install "vanity_crypto-macos-arm64" => "vanity_crypto" if OS.mac?
    bin.install "vanity_crypto-linux-amd64" => "vanity_crypto" if OS.linux?
  end

  test do
    assert_match "Usage:", shell_output("#{bin}/vanity_crypto --help")
  end
end
