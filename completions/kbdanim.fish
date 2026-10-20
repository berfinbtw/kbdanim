# Fish completion for kbdanim.
#
# Install: copy to ~/.config/fish/completions/kbdanim.fish

set -l kbdanim_commands run daemon stop status restart list animation search color brightness mode static off gaming night preset profile power-profile random-mode playlist interactive doctor config completions
set -l kbdanim_animations static breathing pulse heartbeat strobe rainbow smooth-rainbow color-cycle random random-pastel breathing-color warm cool night fire candle ocean forest aurora sunset lava ice meteor lightning sparkle oscillation wave plasma cyberpunk matrix toxic party police rgb-cycle

complete -c kbdanim -f

complete -c kbdanim -n "not __fish_seen_subcommand_from $kbdanim_commands" -a "$kbdanim_commands"

complete -c kbdanim -n "__fish_seen_subcommand_from mode" -a "$kbdanim_animations"
complete -c kbdanim -l mode -a "$kbdanim_animations" -d "Animation mode"
complete -c kbdanim -l device -d "LED sysfs directory"
complete -c kbdanim -l speed -d "Speed multiplier (0.1-10.0)"
complete -c kbdanim -l fps -d "Frames per second (1-240)"
complete -c kbdanim -l brightness -d "Brightness (0-255)"
complete -c kbdanim -l color -d "Primary color"
complete -c kbdanim -l color2 -d "Secondary color"
complete -c kbdanim -l easing -a "linear smoothstep ease-in ease-out ease-in-out sine quadratic cubic" -d "Easing curve"
complete -c kbdanim -l period -d "Base cycle/pulse duration"
complete -c kbdanim -l randomness -d "Chaos knob (0-1)"
complete -c kbdanim -l saturation -d "Saturation (0-1)"
complete -c kbdanim -l temperature -d "Warm/cool knob (0-1)"
complete -c kbdanim -l intensity -d "Intensity multiplier"
complete -c kbdanim -l palette -a "warm cool fire ocean forest sunset cyberpunk pastel monochrome neon" -d "Named palette"
complete -c kbdanim -l seed -d "RNG seed"
complete -c kbdanim -s v -l verbose -d "Increase log verbosity"
complete -c kbdanim -s q -l quiet -d "Only log errors"

complete -c kbdanim -n "__fish_seen_subcommand_from completions" -a "bash zsh fish"
complete -c kbdanim -n "__fish_seen_subcommand_from preset" -a "list run create"
complete -c kbdanim -n "__fish_seen_subcommand_from profile" -a "list switch"
complete -c kbdanim -n "__fish_seen_subcommand_from power-profile" -a "ac battery"
complete -c kbdanim -n "__fish_seen_subcommand_from random-mode" -a "on off"
complete -c kbdanim -n "__fish_seen_subcommand_from config" -a "show path edit"
complete -c kbdanim -n "__fish_seen_subcommand_from playlist" -a "run"
