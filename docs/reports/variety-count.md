# Variety count

How many distinct houses can the generator produce? Counted honestly:
the product of the independent decisions, each taken at its minimum
over the six styles, so the bound holds for every style. Decisions
that only increase the count (porch, garage presence, the second
floor, the later splits, the further doors and windows) are left out
of the product — the bound below is a floor, not an estimate.

## The independent decisions

| Decision | Minimum | Why this is the minimum |
|---|---:|---|
| Style | 6 | Six styles, each a real bias. |
| Footprint shape | 1 | Townhouse has one shape; the others have 2–3. |
| Footprint dimensions (rectangle, 0.1 m grid) | 861 | Townhouse 6.0–8.0 m × 12.0–16.0 m = 21 × 41 values. The smallest range. |
| Room count | 2 | Cottage 4–5 rooms; every other range is wider. |
| First split: axis × cut position | 40 | The shortest wall is 7.0 m (Cottage depth): cut positions on the 0.1 m grid over [2.4, 4.6] give 23 positions, times 2 axes = 46; rounded down to 40. |
| First split door position | 50 | Its overlap is the full wall (≥ 7.0 m), so the 0.9 m door slides over ≥ 5.5 m of slack: 56 positions on the 0.1 m grid; rounded down to 50. |
| Entry door position | 8 | The entry room is ≥ 2.4 m wide; the 1.0 m door slides over ≥ 0.8 m of slack: 9 positions; rounded down to 8. |
| Window widths | 27 | The narrowest range is Cottage 0.8–1.0 m = 3 values on the 0.1 m grid; a smallest house has 3 habitable rooms, so 3³. |

## The product

```
6 × 861 × 2 × 40 × 50 × 8 × 27 = 4,463,424,000
```

**≈ 4.5 × 10⁹ distinct houses**, from the most conservative factors
alone. The requirement was 1,000,000; the floor clears it by a
factor of ~4,500.

## What is deliberately NOT counted

- **Program assignment.** It is a deterministic function of the
  leaves (largest leaf near the entry is living, and so on), so it
  contributes no independent variety. Counting it would be double
  counting the leaves. The room-size caps (bath 6 sq m, study
  12 sq m) are likewise deterministic — they split or reassign
  leaves by rule — so they do not change the numbers above. M0
  re-tiling (alcove subdivision, bounded bath count, exterior
  preference for habitable rooms) is also deterministic: it
  redraws leaf boundaries by rule and re-pins rects until the
  caps hold, so it adds no independent decision either. The
  re-tiling changed which plan a seed produces, not how many
  plans the decisions can produce; the empirical check below
  still gives 1000 distinct hashes over 1000 seeds.
- **Porch** (present/absent for Ranch, Colonial, Cottage), **garage**
  (present/absent plus its slide position for Ranch, Colonial,
  Craftsman), the **second floor** (its own room count, split tree,
  doors and windows for Colonial and Townhouse), the **later splits**
  (each with its own axis and cut position), and the **further doors
  and windows** (each with its own width and position). All of these
  multiply the count further.
- **The seed space.** The generator takes a u32 seed, so at most
  2³² ≈ 4.3 × 10⁹ distinct houses can ever be produced. That is the
  binding cap, and it is still ~4,300× the requirement. The decision
  space above (≈ 4.5 × 10⁹) is of the same order as the seed space,
  which is why the empirical check is clean.

## Empirical check

1000 consecutive seeds (cycling through all six styles) produce
**1000 distinct plan hashes**. The hash is FNV-1a over the seed-free
plan JSON (`render_plan_json`), so distinct hashes mean distinct
plans rather than distinct seeds. See `tests/generation.rs`.
