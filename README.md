# wslwrap

A way to symlink [Windows Subsystem for Linux](https://docs.microsoft.com/en-us/windows/wsl/faq) programs onto the PATH for Windows.

## Status

![Azure DevOps builds](https://img.shields.io/azure-devops/build/TheGoddessInari/332a0b97-a83e-434c-89d7-6bac0fcfb508/3.svg?logo=azure-devops)

![Azure DevOps releases](https://img.shields.io/azure-devops/release/TheGoddessInari/332a0b97-a83e-434c-89d7-6bac0fcfb508/1/1.svg?logo=azure-devops)

## Example

Example usage:

Put it in a unique directory on the PATH.

```cmd
D:\wslwrap>mklink uname.exe wslwrap.exe
symbolic link created for uname.exe <<===>> wslwrap.exe

D:\wslwrap>uname -a
Linux HOSTNAME  4.4.0-17763-Microsoft #379-Microsoft Wed Mar 06 19:16:00 PST 2019 x86_64 x86_64 x86_64 GNU/Linux

D:\wslwrap>mklink ls.exe wslwrap.exe
symbolic link created for ls.exe <<===>> wslwrap.exe


D:\wslwrap>ls /'Program Files'/'Windows NT'/
accessories  tabletextservice
```

This should also work with varying path separators and UNIXisms like / (root of current drive), and ~ (USERPROFILE).
