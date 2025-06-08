# factory
![factory](/assets/trmt_v0_3_0_factory_example.png)

### Config
```toml
[simulation]
heads = 4
rule = "R1>1,L0>1:U1>0,D0>1"
speed_ms = 5.0
trail_length = 24
color_cells = true
seed = "xeumfqni"

[display]
colors = ["#ff9a8b", "#ffecd2", "#a8edea", "#fed6e3"]
state_based_colors = false
live_colors = false
head_char = ["██"]
trail_char = ["▓▓"]
cell_char = "░░"
```