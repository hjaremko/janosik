# Janosik ![Build](https://github.com/hjaremko/janosik/workflows/Build/badge.svg?branch=master)
Janosik is a Discord bot helping Jagiellonian University computer science students
test their programming assignments.

Currently Janosik works by running pre-installed correct program in your system shell.
**This may not be secure.** 

### Running
```
JANOSIK_TOKEN=<your Discord bot token> cargo run --release
```

### Usage
Janosik responds to commands on public channels as well as private messages.

```
!blackbox <binary filename>
    ```
    input
    ```
```
`input` will be redirected to the standard input of the binary.

#### Example
![Gra towarzyska](https://i.imgur.com/3v3JAsw.png)

### Setting up tasks
Janosik searches for binary files in the `bin` directory of the project root.
Then, the program can be called by its filename.

### Downloads

Latest releases are available [here](https://github.com/hjaremko/janosik/releases).
