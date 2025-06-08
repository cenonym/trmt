# dotgrid
![dotgrid](/assets/trmt_v0_3_0_dotgird_example.png)

### Config
```toml
[simulation]
heads = 6
rule = "R1>1,L0>2,U1>0:D0>0,R1>1:L0>1,R1>2"
speed_ms = 30.0
trail_length = 256
color_cells = false
seed = "8mcoh0xa"

[display]
colors = ["#264653", "#2a9d8f", "#e9c46a", "#f4a261"]
fade_trail_color = ""
state_based_colors = false
live_colors = false
randomize_heads = false
randomize_trails = false
head_char = ["⬤"]
trail_char = ["●","⊛","⊛","○","○","•","•","·","·","·"]
cell_char = "·"
```