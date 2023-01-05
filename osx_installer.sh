# NOAA APT - MacOS Installer Script

bold=$(tput bold)
white='\e[1;37m'
clean='\e[0m'

printf "${bold} --- NOAA APT's Installer for OSX ---\n"

brew_check=$(brew -v 2> /dev/null | head -n 1 | grep Homebrew -c)

if [ ${brew_check} = "0" ]
then
printf "\n${white} [?] You need to install Homebrew (Package Manager for MacOS)... Install it now? (may take some time) [Y/n]: ${clean}"
read -p "" brew_install_choice
if [ ${brew_install_choice} = "Y" ] || [ ${brew_install_choice} = "y" ]
then
echo -e "\n"
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
else
printf "\nScript is quiting..."
exit
fi
fi

printf "\n${white} [*] Installing the newest version of Rustup${clean}"
echo -e "\n"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

printf "\n${white} [*] Installing dependencies via Homebrew (gtk+3 adwaita-icon-theme openssl)${clean}"
echo -e "\n"
brew install gtk+3 adwaita-icon-theme openssl

printf "\n${white} [*] Compiling the source code...${clean}"
echo -e "\n"
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig/
cargo build --release

printf "\n${white} [*] Installation completed... Run it with: ./target/release/noaa-apt${clean}\n"
