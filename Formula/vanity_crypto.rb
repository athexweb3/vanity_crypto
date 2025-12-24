class VanityCrypto < Formula
  desc "High-performance, secure, and beautiful Ethereum vanity address generator"
  homepage "https://github.com/athexweb3/vanity_crypto"
  version "0.3.0"
  
  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v#{version}/vc-macos-arm64"
    sha256 "ee88b3c2ae1faf4756688e9c90b7ab25b369e63722930da533e015922734d967" # MAC_ARM_SHA
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v#{version}/vc-macos-amd64"
    sha256 "2553d7fb5a1e6d806bf2d7ffbcbed6f821581fea66aedc499de2dede0a89e1ef" # MAC_INTEL_SHA
  elsif OS.linux?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v#{version}/vc-linux-amd64"
    sha256 "370b79fcbdfcac42fd0b62769fc2964faa8795e71111104c37236640e6ac9309" # LINUX_SHA
  end

  def install
    if OS.mac? && Hardware::CPU.arm?
      bin.install "vc-macos-arm64" => "vc"
    elsif OS.mac? && Hardware::CPU.intel?
      bin.install "vc-macos-amd64" => "vc"
    elsif OS.linux?
      bin.install "vc-linux-amd64" => "vc"
    end
  end

  test do
    assert_match "Usage:", shell_output("#{bin}/vc --help")
  end
end
