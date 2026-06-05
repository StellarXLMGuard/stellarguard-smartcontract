# Maintainers — StellarGuard

This document defines the governance structure, maintainer roles, code review processes, and security procedures for the StellarGuard project.

## Core Maintainers

| Name | GitHub | Role | Area of Responsibility |
|------|--------|------|----------------------|
| Project Lead | @stellar-guard | Technical Lead | Overall architecture, smart contracts, technical direction |
| TBD | @tbd | Backend Lead | API server, database, event listener, infrastructure |
| TBD | @tbd | Frontend Lead | Next.js application, UI/UX, user experience |
| TBD | @tbd | Security Lead | Security audits, vulnerability management, security policies |

## Role Definitions

### Technical Lead
- Sets technical direction and architectural decisions
- Reviews and approves major feature implementations
- Coordinates between backend and frontend teams
- Ensures code quality and best practices across the codebase
- Acts as final decision-maker for technical disagreements

### Backend Lead
- Oversees API server development and maintenance
- Manages database schema and migrations
- Responsible for event listener reliability
- Ensures backend performance and scalability
- Reviews all backend pull requests

### Frontend Lead
- Oversees Next.js application development
- Ensures responsive design and accessibility
- Manages UI component library and design system
- Optimizes frontend performance and user experience
- Reviews all frontend pull requests

### Security Lead
- Conducts security audits and penetration testing
- Manages vulnerability disclosure process
- Reviews security-related pull requests
- Maintains security documentation and policies
- Coordinates incident response for security issues

## Code Review Assignment Rules

### Automatic Assignment
- **Backend PRs**: Automatically assigned to @tbd (Backend Lead)
- **Frontend PRs**: Automatically assigned to @tbd (Frontend Lead)
- **Smart Contract PRs**: Automatically assigned to @stellar-guard (Technical Lead)
- **Security-related PRs**: Automatically assigned to @tbd (Security Lead) + relevant domain lead

### Review Requirements
- **At least one approval** from the assigned domain lead is required for merge
- **Security-sensitive changes** require approval from both domain lead and Security Lead
- **Breaking changes** require approval from Technical Lead
- **Documentation updates** require review from at least one maintainer

### Review SLA
- Maintainers should respond to review requests within **48 hours**
- If a maintainer is unavailable, they should assign a proxy reviewer
- Emergency security fixes bypass normal SLA and should be reviewed immediately

### Review Criteria
- Code follows project style guidelines (see `STYLE.md`)
- Changes include appropriate tests
- Documentation is updated for API/user-facing changes
- No security vulnerabilities introduced
- Performance impact is considered
- Error handling is robust

## Security Contact Procedures

### Reporting Security Vulnerabilities
**Do NOT** open public issues for security vulnerabilities.

1. **Email**: Send detailed reports to `security@stellarguard.io`
2. **Include**: Steps to reproduce, impact assessment, and suggested fix
3. **Response Time**: Security team will acknowledge within 48 hours
4. **Disclosure**: Coordinated disclosure timeline will be established

### Security Team
- **Primary**: @tbd (Security Lead)
- **Secondary**: @stellar-guard (Technical Lead)
- **Escalation**: For urgent issues, contact via GitHub security advisories

### Incident Response
1. **Assessment**: Security team assesses severity (CVSS score)
2. **Mitigation**: Develop and test fix in private fork
3. **Coordination**: Notify affected parties if necessary
4. **Disclosure**: Public disclosure after fix is deployed
5. **Post-mortem**: Document incident and improve processes

### Security Policy
- See `SECURITY.md` for detailed security policies
- Security-related PRs should be marked with the `security` label
- All dependencies are scanned for vulnerabilities via Dependabot
- Security audits are conducted annually or before major releases

## How to Become a Maintainer

### Criteria
1. Consistently contribute high-quality PRs over 3+ months
2. Demonstrate deep understanding of the codebase
3. Actively review other contributors' code
4. Participate in discussions and issue triage
5. Show leadership in community discussions

### Process
1. Self-nomination or nomination by existing maintainer
2. Review of contributions by current maintainers
3. Discussion in maintainer meeting
4. Consensus approval required
5. Announcement to community

### Maintainer Removal
Maintainers may be removed for:
- Inactivity (no contributions for 6+ months without notice)
- Repeated violation of code of conduct
- Consistent failure to meet review SLAs
- Other reasons determined by maintainer consensus

## Maintainer Responsibilities

### Code Review
- Review and merge pull requests within 48 hours
- Provide constructive, actionable feedback
- Ensure all tests pass before merging
- Verify documentation is updated

### Issue Triage
- Review new issues within 72 hours
- Assign appropriate labels and priority
- Route issues to appropriate maintainers
- Close stale or duplicate issues

### Documentation
- Keep documentation up-to-date with code changes
- Review and improve documentation PRs
- Ensure API documentation is accurate
- Maintain architecture diagrams

### CI/CD
- Monitor CI/CD pipeline health
- Fix failing builds promptly
- Ensure deployment processes are smooth
- Review and improve CI/CD configurations

### Community
- Mentor new contributors
- Answer questions in discussions
- Foster inclusive community environment
- Represent project in external forums

### Meetings
- Attend bi-weekly maintainer meetings
- Provide status updates on area of responsibility
- Participate in decision-making discussions
- Document action items and follow through

## Decision Making

### Consensus Model
- Technical decisions require consensus among relevant maintainers
- Technical Lead has tie-breaking authority
- Major changes require discussion in maintainer meeting
- Document decisions in project wiki or relevant issues

### Conflict Resolution
1. Discuss among involved maintainers
2. Escalate to Technical Lead if unresolved
3. If still unresolved, call vote among all maintainers
4. Technical Lead makes final decision if vote is tied

## Communication

### Channels
- **GitHub Issues**: Public discussions and bug reports
- **GitHub Discussions**: Community questions and feature requests
- **Maintainer Meetings**: Private bi-weekly sync (video call)
- **Slack/Discord**: Real-time communication (private maintainer channel)

### Transparency
- Major decisions are documented in public issues
- Meeting notes are shared with maintainers
- Roadmap is publicly visible
- Security disclosures follow coordinated disclosure process
