1. USER INSTRUCTION MUST FOLLOW:
# Context Management
- codex_context.md will contain all details about previous step and keep the context of whole conversation.
- Before you do any task you have to make sure you read codex_context.md before you solve the task.
- After finish the task you MUST summarize all details following this format to codex_context.md file.
```xml
<task_number>
[The task number of the task]
<state_snapshot>
    <overall_goal>
        [Single concise sentence describing the task's high-level objective]
    </overall_goal>

    <key_knowledge>
        [Bullet points of crucial facts, conventions, and constraints]
        - Technical stack and frameworks being used
        - Important configuration or setup details
        - Key decisions made during the task
        - User preferences and requirements
    </key_knowledge>

    <file_system_state>
        [List of files created/modified/deleted with their status]
        - Current working directory
        - Files read and key findings
        - Files modified and nature of changes
        - Files created and their purpose
    </file_system_state>

    <recent_actions>
        [Summary of last few significant actions and outcomes]
        - Commands executed and results
        - Tests run and their status
        - Debugging steps taken
        - Solutions implemented
    </recent_actions>

    <errors_and_warnings>
        [Any unresolved errors or important warnings]
        - Error messages encountered
        - Potential issues identified
        - Warnings to keep in mind
    </errors_and_warnings>

</state_snapshot>
</task_number>
```

Because this file will keep over whole conversation, do not OVERWRITE you need to APPEND to codex_context.md

# Deployment:
- After done a task related to website building you MUST use: bun run dev to deploy and testing before handover.
- You have accessed to playwright MCP to use browing tool, you must use these tool to test full website before you complete.

