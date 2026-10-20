#compdef kbdanim
# Zsh completion for kbdanim.
#
# Install: place this file as `_kbdanim` somewhere on your $fpath (e.g.
# /usr/local/share/zsh/site-functions/_kbdanim), then start a new shell
# (or run `autoload -U compinit && compinit`).

_kbdanim_animations() {
    local -a animations
    animations=(
        static breathing pulse heartbeat strobe rainbow smooth-rainbow
        color-cycle random random-pastel breathing-color warm cool night
        fire candle ocean forest aurora sunset lava ice meteor lightning
        sparkle oscillation wave plasma cyberpunk matrix toxic party
        police rgb-cycle
    )
    _describe 'animation' animations
}

_kbdanim() {
    local -a commands
    commands=(
        'run:Run an animation in the foreground'
        'daemon:Run the animation engine as a foreground daemon'
        'stop:Stop a running daemon'
        'status:Show daemon status'
        'restart:Restart the daemon'
        'list:List all available animations'
        'animation:Animation subcommands'
        'search:Search animations by name or description'
        'color:Set the LED color'
        'brightness:Set the brightness'
        'mode:Change the running daemon animation mode'
        'static:Set a static color'
        'off:Turn the keyboard LED off'
        'gaming:Apply the built-in gaming preset'
        'night:Apply the built-in night preset'
        'preset:Preset subcommands (list/run/create)'
        'profile:Profile subcommands (list/switch)'
        'power-profile:power-profile ac|battery'
        'random-mode:random-mode on|off'
        'playlist:Run a playlist of animations'
        'interactive:Simple terminal-based live control'
        'doctor:Diagnose common setup problems'
        'config:Configuration subcommands (show/path/edit)'
        'completions:Print a shell completion script'
    )

    if (( CURRENT == 2 )); then
        _describe 'command' commands
        return
    fi

    case "${words[2]}" in
        --mode|mode)
            _kbdanim_animations
            ;;
        --easing)
            _values 'easing' linear smoothstep ease-in ease-out ease-in-out sine quadratic cubic
            ;;
        --palette)
            _values 'palette' warm cool fire ocean forest sunset cyberpunk pastel monochrome neon
            ;;
        completions)
            _values 'shell' bash zsh fish
            ;;
        preset)
            _values 'preset command' list run create
            ;;
        profile)
            _values 'profile command' list switch
            ;;
        power-profile)
            _values 'power profile' ac battery
            ;;
        random-mode)
            _values 'state' on off
            ;;
        config)
            _values 'config command' show path edit
            ;;
        *)
            _files
            ;;
    esac
}

_kbdanim
