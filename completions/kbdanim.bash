# Bash completion for kbdanim.
#
# Install: copy this file to /etc/bash_completion.d/kbdanim, or source
# it from your ~/.bashrc: `source /path/to/kbdanim.bash`.

_kbdanim_completions() {
    local cur prev commands flags animations
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    commands="run daemon stop status restart list animation search color brightness mode static off gaming night preset profile power-profile random-mode playlist interactive doctor config completions --help --version"
    flags="--mode --device --speed --fps --brightness --color --color2 --easing --period --randomness --saturation --temperature --intensity --palette --seed --verbose --quiet"
    animations="static breathing pulse heartbeat strobe rainbow smooth-rainbow color-cycle random random-pastel breathing-color warm cool night fire candle ocean forest aurora sunset lava ice meteor lightning sparkle oscillation wave plasma cyberpunk matrix toxic party police rgb-cycle"

    case "$prev" in
        --mode)
            COMPREPLY=($(compgen -W "$animations" -- "$cur"))
            return 0
            ;;
        --easing)
            COMPREPLY=($(compgen -W "linear smoothstep ease-in ease-out ease-in-out sine quadratic cubic" -- "$cur"))
            return 0
            ;;
        --palette)
            COMPREPLY=($(compgen -W "warm cool fire ocean forest sunset cyberpunk pastel monochrome neon" -- "$cur"))
            return 0
            ;;
        completions)
            COMPREPLY=($(compgen -W "bash zsh fish" -- "$cur"))
            return 0
            ;;
        preset)
            COMPREPLY=($(compgen -W "list run create" -- "$cur"))
            return 0
            ;;
        profile)
            COMPREPLY=($(compgen -W "list switch" -- "$cur"))
            return 0
            ;;
        power-profile)
            COMPREPLY=($(compgen -W "ac battery" -- "$cur"))
            return 0
            ;;
        random-mode)
            COMPREPLY=($(compgen -W "on off" -- "$cur"))
            return 0
            ;;
        playlist)
            COMPREPLY=($(compgen -W "run" -- "$cur"))
            return 0
            ;;
        config)
            COMPREPLY=($(compgen -W "show path edit" -- "$cur"))
            return 0
            ;;
        mode)
            COMPREPLY=($(compgen -W "$animations" -- "$cur"))
            return 0
            ;;
    esac

    if [[ "$cur" == -* ]]; then
        COMPREPLY=($(compgen -W "$flags" -- "$cur"))
        return 0
    fi

    if [[ "$COMP_CWORD" -eq 1 ]]; then
        COMPREPLY=($(compgen -W "$commands" -- "$cur"))
        return 0
    fi
}

complete -F _kbdanim_completions kbdanim
