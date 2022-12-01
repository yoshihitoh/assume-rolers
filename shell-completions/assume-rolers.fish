complete -c assume-rolers -a '(assume-rolers -l)'
complete -c assume-rolers -s t -l token -f -r -d 'Specify a token code provided by the MFA device.'
complete -c assume-rolers -s p -l plugin -f -r -a 'export federation (__fish_complete_suffix --or-files .wasm)'
complete -c assume-rolers -x -s l -l list -d 'Show available profiles.'
