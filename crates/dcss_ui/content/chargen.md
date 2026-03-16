---
page:
  name: Chargen
  label: Character Creation
  default: true
widgets:
  species_list:
    max_height: 300.0
  job_list:
    max_height: 300.0
  start_game:
    track_hover: true
---

# Create Your Character {title}

| Species | Background |
|---------|-----------|
| [select](chosen_species){species_list} | [select](chosen_job){job_list} |

---

## [display](preview_name)

| Stat | Value |
|------|-------|
| **Str** | [display](preview_str) |
| **Int** | [display](preview_int) |
| **Dex** | [display](preview_dex) |
| **Skills** | [display](preview_skills) |

[button.primary](Start_Game){start_game}
