# Verification Policy

This document defines the verification and quality assurance policy for the project.

## Quality Standards

The project operates at **standard** tier, requiring:
- All CI checks GREEN
- Lint compliance
- Tests passing
- Security scan clean

## Quality Gates

### Must Pass Gates
1. **Format Check**: Code is properly formatted
2. **Lint**: All linter rules pass
3. **Tests**: All tests pass
4. **Security**: No critical vulnerabilities

### Should Pass Gates
1. **Coverage**: >= 80% code coverage
2. **Documentation**: All public APIs documented
3. **Types**: Type checking passes

## Dispute Workflow

If a quality gate decision is believed incorrect:

1. **Document the dispute** in project issue tracker
2. **Provide evidence** supporting the challenge
3. **Await review** from maintainer (within 48 hours)

## Verification Process

### Pre-commit
- [ ] Format check passes
- [ ] Lint passes
- [ ] Unit tests pass

### Pre-merge
- [ ] All CI checks pass
- [ ] Coverage maintained or increased
- [ ] Security scan clean

### Pre-release
- [ ] All gates green
- [ ] Documentation updated
- [ ] Changelog updated

## Policy Updates

This policy may be updated to reflect:
- New quality standards
- Tooling improvements
- Process refinements

All policy changes require maintainer review.

---

**Last Updated**: 2026-04-02
