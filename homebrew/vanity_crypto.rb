class VanityCrypto < Formula
  desc "High-performance, secure, and beautiful Ethereum vanity address generator"
  homepage "https://github.com/athexweb3/vanity_crypto"
  version "0.1.0-beta.2"
  
  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v#{version}/vc-macos-arm64"
    sha256 "3ba6f6cefd43e6eb3551d384bd6df703ba4545896ef5ccaee4c7215ed664f13a" # MAC_ARM_SHA
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v#{version}/vc-macos-amd64"
    sha256 "f81bb09f83fae62cd35176541ba493dc4de7d67cc1091f3dc086f0c9a000e891" # MAC_INTEL_SHA
  elsif OS.linux?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v#{version}/vc-linux-amd64"
    sha256 "4ecf4d2e5c4de6a379576b5ddd44a5cc62473da89cf4e1713ee5c7e540e91c96" # LINUX_SHA
  end

  def install
    bin.install "vc-macos-arm64" => "vc" if OS.mac?
    bin.install "vc-linux-amd64" => "vc" if OS.linux?
  end

  test do
    assert_match "Usage:", shell_output("#{bin}/vc --help")
  end
end
