<div align="center">
    <h1><span style="color:#f17136;">▜▘▞▘▞▄▚▝▛</span></h1>
    <p>
        2D Turing machine for your terminal.
    </p>
    <a href="https://crates.io/crates/trmt"><img alt="crates.io" src="https://img.shields.io/crates/v/trmt?color=%23f17136"></a>
    <a href="https://github.com/cenonym/trmt/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/badge/License-GPLv3-%23f17136.svg"></a>
    <br>
    <br>
    <img src="/trmt_demo.gif" alt="trmt demo">
</div>

---

### About
[Turmites](https://en.wikipedia.org/wiki/Turmite) are cellular automata where agents, or heads, move on a 2D grid, changing cell states based on rules. Each head reads the current cell, writes a new state, moves in a set direction, then advances.

**trmt** simulates multiple turmites simultaneously with customizable rules, colors, and characters, and has support for both [relative and absolute](https://en.wikipedia.org/wiki/Turmite#Relative_vs._absolute_turmites) turmites.
<br>

### Features
- Up to 256 simultaneous heads
- Support for 16/256-color palettes and true color (RGB/hex)
- Deterministic seed-based simulation for reproducible patterns
- Highly configurable display, controls, and simulation parameters
- Several rule formats for various degrees of complexity
- Real-time interaction with configurable keybinds
- Toroidal grid with seamless wrapping
<br>

### Installation
**trmt** only requires [Rust with Cargo](https://www.rust-lang.org/learn/get-started) as a prerequisite.
> [!NOTE]  
> Pre-compiled binaries and OS/distro-specific managers will be added.


**From crates.io**
```bash
cargo install trmt
```
<br>

**Install from source**
```bash
git clone https://github.com/cenonym/trmt
cd trmt
cargo install --path .
```
<br>

### Usage
Simply run `trmt` in your terminal of choice.
```bash
trmt
```
While **trmt** will work in any terminal *(I think)*, the best performance was observed in [Alacritty](https://github.com/alacritty/alacritty) during testing.
<br>

#### Controls
**trmt** has controls that can be used while the simulation is running. Keybinds can all be customized through the config file, but the defaults are:
| Key | Action |
|:----|:-------|
| `Space` | Pause/resume simulation |
| `q` | Quit |
| `r` | Reset simulation |
| `h` | Toggle help overlay |
| `b` | Toggle statusbar overlay |
| `+` | Increase simulation speed |
| `-` | Decrease simulation speed |
| `c` | Hot reload config (will also reset simulation) |
| `s` | Toggle seed, this will save the seed of your current simulation to your config, or remove it if already present. |
| `1-9` | Set head count (1, 2, 4, 8, 16, 32, 64, 128, 256) |

<br>

### Configuration
**trmt** automatically generates a default config file on first run, or if no config file is present. The location depends on your system:

- **Unix/Linux**: `$XDG_CONFIG_HOME/trmt/config.toml` or `~/.config/trmt/config.toml`
- **macOS**: `~/Library/Application Support/trmt/config.toml`
- **Windows:** `%APPDATA%\trmt\config.toml`
<br>

#### Example config
```toml
[simulation]
default_heads = 3                   # Number of heads on initialization
default_rule = "RL"                 # Rules for the simulation
default_speed_ms = 20               # Simulation speed in milliseconds
trail_length = 24                   # Number of trail characters following the head
infinite_trail = true               # If true, leaves behind an infinite trail of colored cell chars
seed = ""                           # Empty = random

[display]
colors = [                          # Array of colors mapped to number of heads sequentially, using hex, RGB or 256-colors.
    "rgb(241, 113, 54)",          # If there are more heads than colors, remaining colors are generated.
    "#45a8e9",
    "229",
]
head_char = ["██"]                  # Array of head characters, trmt will cycle through it sequentially per step
trail_char = ["▓▓"]                 # Array of trail characters, where first character is mapped to first trail, and so on
cell_char = "░░"                    # The characters left behind the trail when infinite_trail = true

[controls]
quit = "q"                          # Quit
toggle = " "                        # Pause/resume simulation
reset = "r"                         # Reset simulation
faster = "+"                        # Increase simulation speed
slower = "-"                        # Decrease simulation speed
config_reload = "c"                 # Hot reload config
help = "h"                          # Toggle help overlay
statusbar = "b"                     # Toggle statusbar overlay
seed_toggle = "s"                   # Toggle seed
```
> [!TIP]  
> The seed controls starting position and initial direction of the heads. 

<br>

#### Rules
> [!NOTE]  
> A wiki containing a more in-depth guide to the syntax and rule formats will be added to a wiki.

Rules in **trmt** are what defines how the simulation will play out, and how the heads will behave. **trmt** features a custom rule syntax that allows you to create everything from very basic sequential rules, all the way up to academic-level notation *(don't quote me on this)*.

A rule consists of several states, and a state holds specific instructions on how a head should move. You can theoretically have several hundred states in a rule, but a lot of the most interesting patterns will appear with just 3-5 states in a rule.

**Available states:**
- `L/R` - Turn left/right
- `U` - U-turn (move backward)
- `D` - No turn (don't turn, move forward)
- `N/S/E/W` - Absolute directions
- `NW/NE/SW/SE` - Diagonal directions

The parser has deterministic left-to-right precedence, so `"NE"` always becomes diagonal, never `N+E`.

**Basic sequential format**
Using the directions above, we can create simple sequential rules that move through each state in turn. One of the simplest and most famous turmites is [Langton's Ant](https://en.wikipedia.org/wiki/Langton%27s_ant), which can be easily replicated with **trmt** using just `RL` as a rule.
```toml
default_rule = "RL"
```

Or try something like `WRSWNL` or `RRLL`, or even `RULE` if you're feeling meta.

**Rule operators**
In addition to these basic directional states, **trmt** also has logical operators for more complex rules that enable internal multi-states and explicit state transitions.

- `>` - State transition (e.g., R>1 = turn right, go to state 1)
- `,` - Separates rule combinations for explicit state rules
- `:` - Separates states in multi-state rules

For example, these operators allow us to explicitly define how our heads transition between states. This makes for very advanced possibilities, and lets us directly translate a traditional turmite rule like `{{{1, 8, 1}, {1, 8, 1}}, {{1, 2, 1}, {0, 1, 0}}}` (a generalized Langton's Ant), into a more ~~opinionated~~ readable syntax:
```toml
default_rule = "L1>1,L1>1:R1>1,D0>0"
```
This results in a rule that constructs a [Fibonacci spiral](https://en.wikipedia.org/wiki/Turmite#/media/File:Turmite-181181121010-10211.svg).
<br>

### Planned
- [ ] Improved error handling and printing
- [ ] Clean up reset and config reload functions
- [ ] Per-state color customization
- [ ] Customizable initial head direction
- [ ] Toggleable random characters for both heads and trails
- [ ] Redesign help and statusbar TUI
- [ ] Support for 32-bit colors
- [ ] Proper wiki/documentation


### Contributing
You are very welcome to contribute to the project, be that through feature requests, improvements to the code or by adding functions. Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/trmt-feature`)
3. Make your changes
4. Run tests: `cargo test`
5. Check for issues: `cargo clippy` (Warnings can be fine, just no errors)
6. Commit your changes (`git commit -m 'Add trmt feature'`)
7. Push to your branch (`git push origin feature/trmt-feature`)
8. Open a Pull Request

**Found a bug?** Open an [issue](https://github.com/cenonym/trmt/issues) with details and steps to reproduce. **Questions or suggestions?** Let's [discuss](https://github.com/cenonym/trmt/discussions) them.
<br>