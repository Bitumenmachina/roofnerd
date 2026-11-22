# The sweep: one person looking at the building (call 174)

BuildOS is a free, 3D-first, renovation-first estimating program for the whole house:
it takes in what a person has (a PDF plan, a designer's model, a LiDAR scan), builds the
house in 3D, prices it in sixteen divisions, and lists what must be checked in the field.
One building; every trade looks at the same one and switches on what is theirs. You are
ONE of the people who opens it (your role is in your prompt). Review it **as that
person**, plainly and bluntly. You find; you do not decide, and you change no file in
the repository.

## What you have
- The repository at `/home/user/roofnerd` (READ ONLY for you; another agent is working
  in it). The program lives in `plumbline/` (its old working name; the program is
  BuildOS). Read `CLAUDE.md` at the root for what it is meant to be.
- **Pictures of the window as it is**, in `/home/user/roofnerd/plumbline/demos/captured/`
  (`README.md` there says what each folder is). Start with `trades/` (one picture per
  trade, see-through), `house/`, `layers/`, `glass/`, `d2/`, `renow/`, `scanw/`,
  `cabinetsw/`, `layoutsw/`, `roofw/`, `foundw/`, `money/`, `price/`, `field/`, `clash/`,
  `plan/`, `first/`. **Read the pictures themselves** (open the PNG files and look), as
  many as bear on your role: at least 25. Say which you read.
- **The program at the command line** (a frozen copy; it changes no file of the
  repository): `B=/tmp/claude-0/-home-user-roofnerd/ca6a484f-c0e5-523e-99ce-c9d574b09f03/scratchpad/review/bin22/plumbline`.
  `$B` alone lists every verb. A made-up whole house, priced in all sixteen divisions:
  `mkdir -p <your folder> && $B sixteen <your folder>/h.json`, then `$B estimate`,
  `$B vif`, `$B clashes`, `$B takeoff`, `$B finishes`, `$B check` on that file; try what
  your role would try. Work only in a folder of your own under
  `/tmp/claude-0/-home-user-roofnerd/ca6a484f-c0e5-523e-99ce-c9d574b09f03/scratchpad/review/sweep/<your role>/`.
- What was already said about how each trade would take off with it (do not repeat it;
  build on it): `plumbline/research/trades-use-*.md`, `how-a-renovation-is-done.md`,
  `window-survey.md`; what the program has for each division: `plumbline/divisions/README.md`.
- The web, for what the best tools and the published practice of your role show today.
  Go to the makers' own pages and published standards; never write a claim about another
  product from memory. **Put no person's name, company name, project name or email in
  any search or request.**

## Do NOT
- Start the window (`plumbline-app`) or run any script in `plumbline/demos/`: the
  machine is busy and another agent's timed runs would be spoiled. Do not run cargo.
- Write, move or delete anything under `/home/user/roofnerd`. No git commands that
  change anything.
- If a tool call is refused, do not go round it: note it and carry on.

## What to write
Write as you go (a restart can cut you off) to
`/tmp/claude-0/-home-user-roofnerd/ca6a484f-c0e5-523e-99ce-c9d574b09f03/scratchpad/review/sweep/<your role>.md`:

1. **Who I am and why I open this**: the three or four things this person comes to the
   3D building to see or settle, on a renovation, in their own words.
2. **What I saw**: picture by picture (name the file), what this person reads from it in
   the first five seconds, and what they cannot find. Is the building readable as a
   building? With See through on and my trade switched on, can I read my work, its
   sizes, where it runs, what it crosses? What words on screen are not my trade's words?
3. **What is missing or wrong for me**, each as one line: what I looked for, what I got,
   why it matters on a job (money, a failed inspection, a callback, a change order).
4. **What the best tools of my trade show today** (with the source page for each claim),
   and what none of them do that this program could: where it could be ahead, not level.
5. **My ten, in order**: the ten changes that would make this person open it every day,
   most valuable first; each with what the screen would show (be exact: what is drawn,
   what it is called, what a click does) and how one would know it was done.
6. **What must not change**: what is right as it is, for me.

Use the trade's own words. Made-up numbers only. Be exact about what you saw versus what
you suppose: you looked at pictures and the command line, not the live window; say
"not seen" where a thing may exist but no picture shows it.

Your final message: five lines at most, then the path of your notes.
