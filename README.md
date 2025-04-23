# CHIP-8
A CHIP-8 emulator/interpreter written in Rust using SDL2.

<h2>About</h2>

> CHIP-8 is an interpreted programming language, developed by Joseph Weisbecker on his 1802 microprocessor.
> It was initially used on the COSMAC VIP and Telmac 1800, which were 8-bit microcomputers made in the mid-1970s.
> CHIP-8 was designed to be easy to program for, as well as using less memory than, other programming languages like BASIC.
> 
> _https://en.wikipedia.org/wiki/CHIP-8_

This project has been an insightful introduction to emulation and the way computers worked in the past.
I recommend you try to create your own CHIP-8 emulator, as it can be a small but challenging project for beginners.

<h2>System specifications</h2>
<ul>
  <li>4 KiB memory (4096 bytes)</li>
  <li>simple font with the hexadecimal charcters</li>
  <li>64x32 monochrome (2 color) display</li>
  <li>stack for 16-bit addresses to subroutines and functions</li>
  <li>delay timer, decrements if not 0</li>
  <li>sound timer, same as delay timer, but also beeps if not 0</li>
  <li>16 8-bit variable registers</li>
  <li>program counter (PC), pointing to the current instruction in memory</li>
  <li>index register. pointing at instructions in memory</li>
</ul>


<h2>How to run</h2>

> [!IMPORTANT]
> You will need Rust installed (obviously) and, most importantly, SDL2.
> Without SDL2 the program will refuse to run.

<p>1. Clone this GitHub repository:</p>

```
git clone https://github.com/TAugustL/CHIP-8.git
```

<p>2. Enter the now created folder and enter in your terminal:</p>

```
cargo run --release [path/to/the/chip-8-ROM]
```

<p>3. The emulator should now start with your game. Enjoy!</p>

> [!NOTE]
> CHIP-8 went through some changes during its lifetime.
> Some functions may be handled diffrently than what the supplied ROM may be expecting.
> However, you can switch between these ambiguous functions in the 'Cargo.toml' file by uncommenting the first line under '[features]'.
> By default this emulator uses the modern conventions, so you should not need to change anything.

<h2>Used sources:</h2>
<ul>
  <li><a href="https://tobiasvl.github.io/blog/write-a-chip-8-emulator/">This guide by Tobias V. Langhoff</a></li>
  <li><a href="https://austinmorlan.com/posts/chip8_emulator/">This tutorial by Austin Morlan</a>  (it's in C++, but still super helpful)</li>
</ul>
