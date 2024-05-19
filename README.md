continuation of https://github.com/JafKC/siglauncher, as I lost my old account

# Minelander
This is Minelander, a Minecraft launcher made with Rust and the Iced GUI library. 
The launcher is compatible with Vanilla, Fabric, and Forge, and is designed to run on both Windows and Linux.

Note: For now the launcher only works in offline mode.


### Features
* Simple and intuitive GUI.
* Version installer.
* Compatibility: works with any vanilla release, Fabric and Forge.
* Instance system: useful for modpacks and for those who play in multiple versions.
* Game performance: optimized Java flags.
* Works in offline mode.
* No need to install Java, the launcher provides both Java 8 and Java 17.

![image](https://github.com/jafkc2/minelander/assets/150557443/80cc2f42-7599-453b-aa1b-436cb1601937)



### Installation
###### Build method
Requires Git and Rust to be installed. Type the following commands:

```bash
git clone https://github.com/jafkc2/minelander.git
```
```bash
cd minelander
```
```bash
cargo build --release
```
The executable will appear inside **target/release**.

###### Release method
Download from [releases](https://github.com/Jafkc2/minelander/releases).


### Mods
For mods, you can choose between [Fabric](https://fabricmc.net/) or [Forge](https://files.minecraftforge.net/net/minecraftforge/forge/). Download mods from [Mondrith](https://modrinth.com/mods) and paste them into the mods folder within your Minecraft directory.

You can download Fabric versions from the launcher. If you want to use Forge then download it from [here](https://files.minecraftforge.net/net/minecraftforge/forge/).

