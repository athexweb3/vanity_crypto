class VanityCrypto < Formula
  desc "High-performance, secure, and beautiful Ethereum vanity address generator"
  homepage "https://github.com/athexweb3/vanity_crypto"
  version "0.2.0"
  
  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v#{version}/vc-macos-arm64"
    sha256 "12115cdb5e3f4fa75460b1cbb8d13904655fd8e0b6cbc75a25150dab034b8403" sha256 "7129201256264acf275bfcc1f2e6d4540c77781727dea2dcc070dc21f30b7bab" sha256 "f7c616202d5f4db8edbd50a174eeb2a4ae61e4d70f21a31915ebe8fd14620140" # MAC_ARM_SHA
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v#{version}/vc-macos-amd64"
    sha256 "25183d805d195f86e0c6e7ddd6f8e3b61b981d8cb58e241ef958a35a1e7fd595" sha256 "3077366944c8ee5ba7622f4b3ae6d29f45c1c2537d91cf7e11f616c90c5660c6" sha256 "bd1e8e381209a89d6498ddda7d823e288e51849962e877bd8a7db7f7af165c7d" # MAC_INTEL_SHA
  elsif OS.linux?
    url "https://github.com/athexweb3/vanity_crypto/releases/download/v#{version}/vc-linux-amd64"
    sha256 "f55317042bf833c6e215bcf9d05df8d784e285393a2c57435a9adfd11e5c500c" sha256 "340e396ff9272dbbdbf9e2c7b84324c59e2948e68979889c8b63824cbde562c7" sha256 "da6e4ca3ddf660430839e727d02a0dea04120c1043634a8dfd7fa07b7b9d7aa2" # LINUX_SHA
  end

  def install
    bin.install "vc-macos-arm64" => "vc" if OS.mac?
    bin.install "vc-linux-amd64" => "vc" if OS.linux?
  end

  test do
    assert_match "Usage:", shell_output("#{bin}/vc --help")
  end
end
