# Setup Simple Commander on Raspberry Pi like devices

## Purpose
I usually setup Java for any device I can reach by my hands to. But some devices are
really small, so I decided to test Rust on them instead.

## Initial setup
Use the

> sudo raspi-config

and in Interfaces section, enable SSH.

If you have USB drives, as I do, you need to mount them first. Issue:

```
root2@pi:~ $ lsblk
NAME        MAJ:MIN RM  SIZE RO TYPE MOUNTPOINTS
sda           8:0    0  3.6T  0 disk 
└─sda1        8:1    0  3.6T  0 part 
mmcblk0     179:0    0 29.7G  0 disk 
├─mmcblk0p1 179:1    0  512M  0 part /boot/firmware
└─mmcblk0p2 179:2    0 29.2G  0 part /
```

Note a name of the device. You need to create a mount point for it first:

> root2@pi:~ $ sudo mkdir /media/usbdrive

A name of a directory can be any not clashing with system directories names, and then issue:

> sudo blkid /dev/sda1

You get a result like:

```
/dev/sda1: LABEL="My Book" BLOCK_SIZE="4096" UUID="A2CA0AEBCA0ABC13" TYPE="ntfs"
```

Now you can edit *fstab*

> sudo nano /etc/fstab

and add a new record as below based on *blkid* information and the created directory,

```
UUID=\[UUID] \[MOUNT POINT] \[TYPE] defaults,auto,users,rw,nofail,noatime 0 0
```

, e.g.

```
UUID=A2CA0AEBCA0ABC13 /media/usbdrive ntfs defaults,auto,users,rw,nofail,noatime 0 0
```
**note**: that `noatime` attribute makes sense for SSD drives only.

## Install Samba
This step is optional and required in a case if you like to share the added drive. Issue the -

> sudo apt install samba samba-common-bin

Now you need to configure it by editing:

> sudo nano /etc/samba/smb.conf

An entry as below can be added:

```
\[your_share_name]
    path = /media/usbdrive
    writeable = yes
    browseable = yes
    public = no
```

*your_share_name* and *path* will depend on your preferences. Unless you set *public* to `yes`, you need also add
a Samba user using:

> sudo smbpasswd -a <USERNAME>

After you applied the changes, restart Samba using:

> sudo systemctl restart smbd


Disclaimer: the section was created with Google AI help

If you need to do some other work on you devices, you can read [the small guide](https://sourceforge.net/p/tjws/git/ci/master/tree/1.x/doc/sbc/README.md).

## Install Rust

Issue the 

> curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

Select a desired installation type.

You will need to relogining in SSH session or restart the terminal after the installation's finished.

## Build Rust crates and final web server with apps

Install git unless you already have it,

> sudo apt install git

Copy you SSH keys (if needed)

> scp ~/.ssh/id_rsa* root2@pi:/home/root2/.ssh/

Create a projects directory, unless you already have some and willing to reuse. The guide will refer the directory
as _projects_ assuming that you can select any other name of your choice.

Clone git repositories to the directory,

- https://gitlab.com/tools6772135/rusthub.git
- https://github.com/vernisaz/simtime.git
- https://github.com/vernisaz/simple\_rust_zip.git

### Install Java

It's required to bootstrap **rb** tool building. 

Run the

> sudo apt install openjdk-17-jdk-headless

### Install 7Bee

You do not need to install the product. The **jar** from it will be enough, therefore go to [the page](https://sourceforge.net/projects/seven-bee/)
and download *7Bee-1.3.1-on-2023-09-29(12 24).zip*. Create the directory *7bee/lib* inside *projects* and extract *bee.jar* there.

### Build rb

Create an empty directory called _crates_ in the **projects**. Navigate to *rusthub/src/rust/rustbee*
and edit *bee-rust.xml* there. You need to modify only,

```
<!ENTITY crates "/Users/root2/projects/crates">
```
specifying an absolute path to directory *crates* you created on the previous step.

Issue,

> java -jar ../../../../7bee/lib/bee.jar

The RustBee tool will be built after. You can install it issuing,

> sudo ./rb install

or simply specify the path to *rb*, if you do not want an installation.

You can also add the path to **rb** in the *PATH* environment variable.

## Building Rust apps
All my Rust applications have a web UI. Therefore, first application will be a web server.

Start with cloning the following repositories in the *projects*,

- [RightSlash](https://github.com/vernisaz/right_slash)
- [SimJSON](https://github.com/vernisaz/simjson)
- [SimThreadPool](https://github.com/vernisaz/simtpool)
- [SimWeb](https://github.com/vernisaz/simweb)

And then run **rb** in every of them. It will build required common crates.

The crates are actually required to build a web server, and can be used in other
applications too.

### SimHTTP

First check out its git [repository](https://github.com/vernisaz/simhttp) in **projects** and then execute **rb** in it.
Do not execute the server yet, because [env.conf](https://github.com/vernisaz/simhttp/blob/master/env.conf) needs
to be modified to specify the server port number and serviced directories.

### Run SimHTTP as a service
It is a convenient to run the server as a service. First,  create a directory where the server will reside. Using a development
location for a system service isn't a good idea. Only two files needed to be copied there, simhttp and env.conf.

Second, you need to edit file [rustcom.service](https://github.com/vernisaz/simcom/blob/master/cfg/rustcom.service) specifying the
selected location for the server.

Last, copy the service file to /usr/lib/systemd/system/, as:

> sudo cp rustcom.service /usr/lib/systemd/system/

The following set of commands is used to control the service,

- enable - `sudo systemctl enable rustcom`
- start - `sudo systemctl start rustcom`
- stop - `sudo systemctl stop rustcom`
- disable - `sudo systemctl disable rustcom`

You can selected any other name for the service not clashing with already existing services.

### A developer paradise

If you do a development using Rust, then I will recommend to install the Rust Development Studio
([RDS](https://sourceforge.net/projects/rustdevelopmentstudio/)), you can download it or build.

Now you can use a web browser to develop, debug, test and run Rust applications and crates.



