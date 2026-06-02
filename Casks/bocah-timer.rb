cask "bocah-timer" do
  version "0.1.0"
  sha256 :no_check # unsigned build; release asset changes per push

  url "https://github.com/pradityaaldi/bocah-timer/releases/download/app-v#{version}/Bocah.Timer_#{version}_universal.dmg"
  name "Bocah Timer"
  desc "Fullscreen countdown overlay timer for macOS"
  homepage "https://github.com/pradityaaldi/bocah-timer"

  app "Bocah Timer.app"

  # brew strips the quarantine flag on install, so no Gatekeeper warning.

  zap trash: [
    "~/Library/Preferences/com.praditya.bocahtimer.plist",
    "~/Library/Application Support/com.praditya.bocahtimer",
    "~/Library/Caches/com.praditya.bocahtimer",
  ]
end
