_fedora-update-feedback() {
    local i cur prev opts cmds
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    opts=""

    for i in ${COMP_WORDS[@]}
    do
        case "${i}" in
            fedora-update-feedback)
                cmd="fedora-update-feedback"
                ;;
            
            *)
                ;;
        esac
    done

    case "${cmd}" in
        fedora-update-feedback)
            opts=" -O -P -c -I -U -i -p -v -h -V -u -A -R  --check-obsoleted --check-pending --check-commented --check-ignored --check-unpushed --clear-ignored --ignore-keyring --print-ignored --verbose --help --version --username --add-ignored-package --remove-ignored-package  "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --add-ignored-package)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --remove-ignored-package)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -R)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        
    esac
}

complete -F _fedora-update-feedback -o bashdefault -o default fedora-update-feedback
