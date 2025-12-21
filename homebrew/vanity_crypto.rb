class VanityCrypto < Formula
  desc "High-performance, secure, and beautiful Ethereum vanity address generator"
  homepage "https://github.com/athexweb3/vanity_crypto"
  version "0.1.0-beta.0"
  
  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v#{version}/vc-macos-arm64"
    sha256 "81f5cd7f27784dc47a3eac839ff3442a3b909a848182da22aca180667fc404b8" # MAC_ARM_SHA
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v#{version}/vc-macos-amd64"
    sha256 "REPLACE_WITH_ACTUAL_SHA256_AFTER_RELEASE_BUILD_MACOS_INTEL" # MAC_INTEL_SHA
  elsif OS.linux?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v#{version}/vc-linux-amd64"
    sha256 "25afe42080d96ca1d710f6f177283a5d12838f284c9aa11a91981930dc131df9" # LINUX_SHA
  end

  def install
    bin.install "vc-macos-arm64" => "vc" if OS.mac?
    bin.install "vc-linux-amd64" => "vc" if OS.linux?
  end

  test do
    assert_match "Usage:", shell_output("#{bin}/vc --help")
  end
end
