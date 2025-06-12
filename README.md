<div align="center">
    <h1><span style="color:#f17136;">▜▘▞▘▞▄▚▝▛</span></h1>
    <p>
        2D Turing machine for your terminal.
    </p>
    <a href="https://crates.io/crates/trmt"><img alt="crates.io" src="https://img.shields.io/crates/v/trmt?color=%23f17136"></a>
    <a href="https://github.com/cenonym/trmt/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/badge/License-GPLv3-%23f17136.svg"></a>
    <br>
    <a href="#installation">Installation</a> | <a href="/examples">Examples</a>
    <br><br>
    <img src="/trmt_demo.gif" alt="trmt demo">
    <p>
        Demo background by <a href="https://silverwing-vfx.de">Raphael Rau</a>
    </p>
</div>

---

### About
**trmt** can simulate multiple turmites simultaneously with customizable rules, colors, and characters, and has support for both [relative and absolute](https://en.wikipedia.org/wiki/Turmite#Relative_vs._absolute_turmites) turmites.

Running **trmt** will start a simulation with the [default config](#configuration).

<br>

### Features
- Full Unicode support
- Up to **256 simultaneous heads**
- **Full color support**: 16-color, 256-color, and RGB/hex
- **Randomized rule generator** with roughly `10^15` possible rules
- **Deterministic seed-based simulation** for reproducible patterns
- **Highly configurable** parameters for simulation, display and keybinds
- **Several rule formats** for various degrees of complexity
- **Real-time interaction** with configurable keybinds
- Toroidal grid with seamless wrapping
<br>

### Installation
**trmt** only requires [Rust with Cargo](https://www.rust-lang.org/learn/get-started) as a prerequisite.

**From crates.io**
```bash
cargo install trmt
```
<br>

**From AUR (Arch Linux)**
```bash
paru -S trmt
```
<br>

**Using Flakes (Nix)**

Add `trmt` to your flake as an input,
```nix
{
  inputs = {
    ... # other inputs
    trmt.url = "github:cenonym/trmt";
  };
}
```

Then, you can add `inputs.trmt.packages.${pkgs.systems}.default` directly into your `environment.systemPackages` or `home.packages`:
```nix
{
  outputs = { nixpkgs, trmt, ... }: {
    nixosConfigurations.<mysystem> = nixpkgs.lib.nixosSystem {
      modules = [
        ({ pkgs, ... }: {
          environment.systemPackages = [ trmt.packages.${pkgs.system}.default ];
        })
      ];
    };
  };
}
```

<details>
<summary>Installing and configuring trmt with home-manager</summary>

You can also use the `home-manager` module to install and configure `trmt`. First, you need to add `inputs.trmt.homeManagerModules.default` to your imports in your `home-manager` config, and then you can directly configure `trmt` (it will create the `$XDG_CONFIG_HOME/.config/trmt/config.toml` file for you):
```nix
{inputs, ...}: {
  imports = [
    inputs.trmt.homeManagerModules.default
  ];

  programs.trmt = {
    enable = true;
    config = {
      simulation = {
        heads = 3;
        rule = "RL";
        speed_ms = 20;
        trail_length = 30;
        color_cells = true;
        seed = "";
      };
      display = {
        colors = ["rgb(241, 113, 54)" "#45a8e9" "229"];
        fade_trail_color = "";
        state_based_colors = false;
        live_colors = false;
        head_char = ["██"];
        trail_char = ["▓▓"];
        cell_char = "░░";
        randomize_heads = false;
        randomize_trails = false;
      };
      controls = {
        quit = "q";
        toggle = " ";
        reset = "r";
        faster = "+";
        slower = "-";
        config_reload = "c";
        help = "h";
        statusbar = "b";
        seed_toggle = "s";
        rule_toggle = "n";
      };
    };
  };
}
```

</details>
<br>

**Install from source**
```bash
cargo install --git https://github.com/cenonym/trmt
```
<br>

### Usage
Simply run `trmt` in your terminal to start a simulation.
```bash
trmt
```

#### Examples
Check out the [examples](/examples) to see some of the possibilities.

https://github.com/user-attachments/assets/eefc272b-09b2-4c9c-93dc-984c1ae60ed0

<br>

#### Controls
**trmt** has controls that can be used while the simulation is running. Keybinds can all be customized through the config file, but the defaults are:
| Key | Action |
|:----|:-------|
| `Space` | Pause/resume simulation |
| `q` | Quit |
| `r` | Reset simulation with current parameters |
| `h` | Toggle help overlay |
| `b` | Toggle statusbar overlay |
| `+` | Increase simulation speed |
| `-` | Decrease simulation speed |
| `c` | Reload from config (clears runtime state) |
| `s` | Generate random seed and reset |
| `n` | Generate random rule and reset |
| `R` | Generate random seed and rule, then reset |
| `1-9` | Set head count (1, 2, 4, 8, 16, 32, 64, 128, 256) |

<br>

### Configuration
**trmt** automatically generates a default config file on first run, or if no config file is present. The location depends on your system:

- **Unix/Linux**: `$XDG_CONFIG_HOME/trmt/config.toml` or `~/.config/trmt/config.toml`
- **macOS**: `~/Library/Application Support/trmt/config.toml`
- **Windows:** `%APPDATA%\trmt\config.toml`
<br>

#### Config options
```toml
[simulation]
autoplay = true                     # If true, simulation starts running automatically on launch, reset and config reload
heads = 3                           # Number of heads on initialization
rule = "RL"                         # Rules for the simulation
speed_ms = 20                       # Simulation speed in milliseconds
trail_length = 24                   # Number of trail characters following the head
color_cells = true                  # If true, leaves behind an infinite trail of colored cell chars
seed = ""                           # Seed for initial position/direction. Empty = random

[display]
colors = [                          # Array of colors mapped to number of heads sequentially, using hex, RGB or 256-colors.
    "rgb(241, 113, 54)",            # If there are more heads than colors, remaining colors are generated.
    "#45a8e9",
    "229",
]
fade_trail_color = ""               # Creates gradient trail from head color to this. Set to terminal bg color to fade out. Empty = no gradient
state_based_colors = false          # Printed cells use color config per state
live_colors = false                 # Heads change color with state, only works with state_based_colors = true
head_char = ["██"]                  # Array of head characters, cycles through it sequentially per step
trail_char = ["▓▓"]                 # Array of trail characters, where first character is mapped to first trail, and so on
cell_char = "░░"                    # The characters left behind the trail when color_cells = true
randomize_heads = false             # Randomize head character in head_char array
randomize_trails = false            # Randomize trail characters in trail_char array

[controls]
quit = "q"                          # Quit
toggle = " "                        # Pause/resume simulation
reset = "r"                         # Reset simulation
faster = "+"                        # Increase simulation speed
slower = "-"                        # Decrease simulation speed
config_reload = "c"                 # Reload config
help = "h"                          # Toggle help overlay
statusbar = "b"                     # Toggle statusbar overlay
seed_toggle = "s"                   # Generate random seed
rule_toggle = "n"                   # Generate random rule
```
> [!NOTE]
> State takes precedence over config and is used across sessions. Use `c` to clear states and reload config defaults, `s`/`n` to generate new random seeds and rules respectively.

<br>

#### Rules
Rules in **trmt** are what defines how the simulation will play out, and how the heads will behave. **trmt** provides you with tools to simulate everything from very basic sequential rules, all the way up to academic-level notation *(don't quote me on this)*.

A rule consists of several states, and a state holds specific instructions on how a head should move when encountered. You can theoretically have several hundred states in a rule, but many of the most interesting patterns will appear with just 3-5 states.

**Available states:**
- `L/R` - Turn left/right
- `U` - U-turn (move backward)
- `D` - No turn (don't turn, move forward)
- `N/S/E/W` - Absolute directions
- `NW/NE/SW/SE` - Diagonal directions

The parser has deterministic left-to-right precedence, so `"NE"` always becomes diagonal, never `N+E`.

**Basic sequential format**
Using the states above, we can create simple sequential rules that move through each state in turn. One of the simplest and most famous turmites is [Langton's Ant](https://en.wikipedia.org/wiki/Langton%27s_ant), which can be replicated with **trmt** using just `RL` as a rule.
```toml
rule = "RL"
```

Or try something like `WRSWNL` or `RRLL`, or even `RULE` if you're feeling meta.

**Rule operators**
In addition to these basic directional states, **trmt** also has logical operators for more complex rules:

- `>` - State transition (e.g. R>1 = turn right, go to state 1)
- `,` - Separates rule combinations for explicit state rules
- `:` - Separates states in multi-state rules

For precise control, you can specify which cell state to write:
- `L1>1` = turn left, write cell state 1, go to state 1
- `R0>0` = turn right, write cell state 0, go to state 0

This lets us translate a traditional turmite notation like `{{{1, 8, 1}, {1, 8, 1}}, {{1, 2, 1}, {0, 1, 0}}}` into a more ~~opinionated~~ readable syntax:
```toml
rule = "L1>1,L1>1:R1>1,D0>0"
```
Which constructs a [Fibonacci spiral](https://commons.wikimedia.org/wiki/File:Turmite-181181121010-10211.png).

**Standard notation support**
**trmt** also supports standard notation for compatibility.
```toml
rule = "{{{1, 8, 1}, {1, 8, 1}}, {{1, 2, 1}, {0, 1, 0}}}"
```

> [!TIP]
> When experimenting with new rules, it is recommended to use `1` head for testing to make the simulation less chaotic.

<br>

### Planned
- [x] Improved error handling and printing - Added in [v0.3.0](https://github.com/cenonym/trmt/releases/tag/v0.3.0)
- [x] Redesign help and statusbar TUI - Added in [v0.3.0](https://github.com/cenonym/trmt/releases/tag/v0.3.0)
- [x] Per-state color customization - Added in [v0.3.0](https://github.com/cenonym/trmt/releases/tag/v0.3.0)
- [x] Toggleable random characters for both heads and trails - Added in [v0.4.0](https://github.com/cenonym/trmt/releases/tag/v0.4.0)
- [x] Gradient trails - Added in [v0.4.0](https://github.com/cenonym/trmt/releases/tag/v0.4.0)
- [x] Clean up reset and config reload functions - Added in [v0.5.0](https://github.com/cenonym/trmt/releases/tag/v0.5.0)
- [x] Random rule generation - Added in [v0.5.0](https://github.com/cenonym/trmt/releases/tag/v0.5.0)
- [ ] Customizable initial head direction
- [ ] Proper wiki/documentation
<br>

### Acknowledgements
A big thanks to:
- [Raphael Rau](https://silverwing-vfx.de) for letting me use his [SLV Console render](https://www.behance.net/gallery/190984217/SLV-Console-%28CGI%29) as the background for the demo gif.
- Developers of [cmatrix](https://github.com/abishekvashok/cmatrix), [pipes.sh](https://github.com/pipeseroni/pipes.sh), [asciiquarium](https://github.com/cmatsuoka/asciiquarium) and the like for inspiring the creation of **trmt**.
- [Ferkel](https://commons.wikimedia.org/wiki/User:Ferkel) on Wikipedia for [turmite rule notations](https://commons.wikimedia.org/wiki/File:Turmite-181181121010-10211.png) used in testing and development of the **trmt** rule syntax.
- [orhun](https://github.com/orhun) for packaging **trmt** on AUR
- [yunusey](https://github.com/yunusey) for adding Nix flake and home-manager support

<br>

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
