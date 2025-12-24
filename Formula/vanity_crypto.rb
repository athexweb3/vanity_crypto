class VanityCrypto < Formula
  desc "High-performance, secure, and beautiful Ethereum vanity address generator"
  homepage "https://github.com/athexweb3/vanity_crypto"
  version "0.3.0"
  
  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v#{version}/vc-macos-arm64"
    sha256 "a93944c0abde883e0aa97074b88a438e4980deacba51caaee110cdc0ccdc4ded" # MAC_ARM_SHA
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v#{version}/vc-macos-amd64"
    sha256 "0806d251106211e101839edf4e7fb3016011c7234d5d4c1fd0947bbcbaeae4b5" # MAC_INTEL_SHA
  elsif OS.linux?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v#{version}/vc-linux-amd64"
    sha256 "96494c66f4c7c9145aa098a4621e3041b328bc46a0e79cbfb8b0338fecd1745e" # LINUX_SHA
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
