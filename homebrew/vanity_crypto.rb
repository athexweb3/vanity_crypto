class VanityCrypto < Formula
  desc "High-performance, secure, and beautiful Ethereum vanity address generator"
  homepage "https://github.com/athexweb3/vanity_crypto"
  version "0.1.0-beta.1"
  
  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v#{version}/vc-macos-arm64"
    sha256 "d28a97aeb9b230e33feede9f3e6428362c3999061acbfbf84a60b8c0e32a4ac2" # MAC_ARM_SHA
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v#{version}/vc-macos-amd64"
    sha256 "494939e743b44c881723d9104ef1838657d8c691512319bbd2fb90befe508239" # MAC_INTEL_SHA
  elsif OS.linux?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v#{version}/vc-linux-amd64"
    sha256 "9846e65f3a7801394cd2e635bdbd2e1a1771fa3e697434657d93fdc537aebbb4" # LINUX_SHA
  end

  def install
    bin.install "vc-macos-arm64" => "vc" if OS.mac?
    bin.install "vc-linux-amd64" => "vc" if OS.linux?
  end

  test do
    assert_match "Usage:", shell_output("#{bin}/vc --help")
  end
end
