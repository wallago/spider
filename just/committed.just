# Verify commit messages follow Conventional Commits
[group('release')]
commits base="main":
    committed -vv $(git merge-base HEAD {{ base }})..HEAD
