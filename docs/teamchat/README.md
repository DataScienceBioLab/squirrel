# Team Communication Guidelines

## Message Format
All team communication files should follow this naming convention:
`TEAM-NNN-type-description.md`

Where:
- `TEAM`: Team identifier (UI, CORE, MCP, DOC)
- `NNN`: Sequential number (001, 002, etc.)
- `type`: Message type (issue, proposal, update, question)
- `description`: Brief description using kebab-case

Example: `UI-001-issue-button-component-regression.md`

## Message Template
```markdown
# [Title]: Clear Description

## From: [Team Name]
## To: [Target Team(s)]
## Date: YYYY-MM-DD
## Status: [Open/In-Progress/Resolved]

### Summary
Brief overview of the communication purpose

### Details
Specific information, including:
- File references with line numbers
- Code snippets
- Screenshots or diagrams if relevant

### Action Items
1. [Action item 1]
2. [Action item 2]
...

### Impact
- [Impact on other teams]
- [Dependencies affected]
- [Performance implications]

### Next Steps
1. [Next step 1]
2. [Next step 2]
...

### References
- Related issues: [links]
- Related rules: [rule references]
- Documentation: [links]
```

## Best Practices
1. One issue/topic per file
2. Always include actionable items
3. Reference specific code locations
4. Link to relevant documentation
5. Keep status updated
6. Use clear, professional language
7. Include all affected teams
8. Provide concrete examples
9. Specify timeline expectations
10. Update with resolution

## Message Types
- `issue`: Report problems or bugs
- `proposal`: Suggest improvements or changes
- `update`: Share progress or changes
- `question`: Request clarification or information

## Status Tracking
- `Open`: Newly created, needs attention
- `In-Progress`: Being addressed
- `Resolved`: Completed and documented

## Cross-Team Dependencies
When referencing other teams' work:
- Link to specific commits
- Reference relevant rules
- Include test results
- Document API changes 