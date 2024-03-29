project_name := "POC"
version := '0.0.1'

pocs := ```ls apps -s | where type == dir | get name  | to text```


# list all recipes
default:
  @just --list

# interactively choose which recipe to run
i:
  @just --choose

# Set nu as the shell and set the table mode to light
set shell := ['nu', '-m', 'light', '-c']

shebang := if os() == 'windows' {
	'nu'
} else {
	'/usr/bin/env nu'
}


run-poc:
    #!{{shebang}}
    let options = ("{{pocs}}" | lines)
    let choice = ($options | input list $"(ansi yellow_italic) Which POC to run?(ansi reset)")
    if ($choice | is-empty) {
       print $"(ansi red)aborting...(ansi reset)"
    } else {
        $env.SLINT_DEBUG_PERFORMANCE = "console,refresh_full_speed"
        $env.SLINT_BACKEND = "winit"
        $env.RUST_LOG = "debug"
        cd $"apps/($choice)"
        cargo run -r
    }