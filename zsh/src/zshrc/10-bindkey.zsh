() {
    # see /etc/zsh/zshrc
    local -A key
    key=(
        BackSpace  "${terminfo[kbs]}"
        Home       "${terminfo[khome]}"
        End        "${terminfo[kend]}"
        Insert     "${terminfo[kich1]}"
        Delete     "${terminfo[kdch1]}"
        Up         "${terminfo[kcuu1]}"
        Down       "${terminfo[kcud1]}"
        Left       "${terminfo[kcub1]}"
        Right      "${terminfo[kcuf1]}"
        PageUp     "${terminfo[kpp]}"
        PageDown   "${terminfo[knp]}"
    )

    bind2maps() {
        local i sequence widget
        local -a maps

        while [[ "$1" != "--" ]]; do
            maps+=( "$1" )
            shift
        done
        shift

        sequence="${key[$1]}"
        widget="$2"

        [[ -z "$sequence" ]] && return 1

        for i in "${maps[@]}"; do
            bindkey -M "$i" "$sequence" "$widget"
        done
    }

    if [[ -n $NMK_TMUX_VERSION ]]; then
        if (( $NMK_TMUX_VERSION >= 2.1 )); then
            _nmk-tmux-copy-mode() tmux copy-mode -eu
        else
            _nmk-tmux-copy-mode() tmux copy-mode -u
        fi
        zle -N _nmk-tmux-copy-mode
        bind2maps emacs         -- PageUp     _nmk-tmux-copy-mode
    else
        bind2maps emacs         -- PageUp     redisplay
    fi
    # press PageDown do nothing
    bind2maps emacs             -- PageDown   redisplay

    unfunction bind2maps
}