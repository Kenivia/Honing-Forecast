
# Coding practices

Always plan and verify the plan with user. Ask necessary clarifying questions.

Keep code comments extremely brief and top-level. Do not document individual input / output.

Never write to ./docs. Never write to project CLAUDE.md. Refer to AI notes for project description. Always read relevant notes before writing code. Maintain these notes as you make changes.

Do not introduce complexity for the sake of completeness. In particular use unwrap liberally in Rust code. Sacrafice complete error messages for simple & brief code. This is not a libary - it just needs to work for our case.

Never commit or merge using git.

Always prefer existing libraries rather than hand-rolling if possible. Search on relevant sites instead of relying on trained memory.

Use tailwind for UI styling. Keep UI styling consistent with the rest of the website. In particular prefer using existing colors in one of the .css files and keep things simple.
