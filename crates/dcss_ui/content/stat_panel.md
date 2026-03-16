---
page:
  name: StatPanel
  label: Stats
widgets:
  hp_bar:
    fill: "#8B0000"
  mp_bar:
    fill: "#1E1E64"
---

# Stats

::: style hp_style
**HP:** [display](hp_text)
:::
[progress](hp_frac){hp_bar}

::: style mp_style
**MP:** [display](mp_text)
:::
[progress](mp_frac){mp_bar}

---

| | |
|---|---|
| **AC** | [display](stat_ac) |
| **EV** | [display](stat_ev) |
| **Str** | [display](stat_str) |
| **Int** | [display](stat_int) |
| **Dex** | [display](stat_dex) |
| **XL** | [display](stat_xl) |
| **XP** | [display](stat_xp) |
| **Gold** | [display](stat_gold) |

::: if has_orb
**ORB OF ZOT** ::gold
:::
