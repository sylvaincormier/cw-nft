def remove_rust_comments(filename):
    with open(filename, 'r') as f:
        lines = f.readlines()

    new_lines = []
    in_block_comment = False

    for line in lines:
        if '/*' in line:
            in_block_comment = True
            line = line.split('/*')[0]

        if '*/' in line:
            in_block_comment = False
            line = line.split('*/')[-1]

        if '//' in line and not in_block_comment:
            line = line.split('//')[0]

        if not in_block_comment:
            new_lines.append(line.rstrip() + '\n')

    with open(filename, 'w') as f:
        f.writelines(new_lines)

# Uncomment this line and provide a Rust file path to run the function
remove_rust_comments('contracts/song_contract/src/lib.rs')


