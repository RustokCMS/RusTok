# Pull Request

## 📋 Description

<!-- Provide a brief description of the changes -->

## 🎯 Related Issues

<!-- Link to related issues, e.g., Fixes #123 -->

## 🔍 Type of Change

<!-- Mark the appropriate option with an 'x' -->

- [ ] 🐛 Bug fix (non-breaking change which fixes an issue)
- [ ] ✨ New feature (non-breaking change which adds functionality)
- [ ] 💥 Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] 📝 Documentation update
- [ ] 🔧 Configuration change
- [ ] ♻️ Code refactoring
- [ ] 🎨 UI/UX improvement
- [ ] ⚡ Performance improvement
- [ ] 🧪 Test addition or improvement

## ✅ Checklist

### Code Quality
- [ ] My code follows the project's coding style
- [ ] I have performed a self-review of my code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] My changes generate no new warnings or errors
- [ ] I have removed any debugging code or console.log statements

### Testing
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] I have tested edge cases and error conditions
- [ ] Test coverage has not decreased

### Documentation
- [ ] I have made corresponding changes to the documentation
- [ ] I have updated the README if needed
- [ ] I have added/updated inline code comments
- [ ] I have added/updated examples if needed

### Security & Validation
- [ ] All domain services use `TransactionalEventBus` (not direct `EventBus`)
- [ ] User input is properly validated and sanitized
- [ ] No SQL injection vulnerabilities
- [ ] No XSS vulnerabilities
- [ ] Sensitive data is not logged
- [ ] Authentication/authorization is properly implemented

### Database & Events
- [ ] Database migrations are included if schema changes were made
- [ ] Events are validated before publishing (using `ValidateEvent`)
- [ ] Events are published within transactions (using `publish_in_tx`)
- [ ] No events are lost due to transaction rollbacks

### Performance & Reliability
- [ ] No potential memory leaks
- [ ] No unbounded resource usage
- [ ] Proper error handling is in place
- [ ] Logging is appropriate (not too verbose, not too sparse)
- [ ] Backpressure/rate limiting considered if applicable

### Backwards Compatibility
- [ ] My changes are backwards compatible
- [ ] If breaking changes exist, I have documented the migration path
- [ ] API versioning is maintained if applicable

## 📊 Test Results

<!-- Paste relevant test results or link to CI -->

```
# Example:
$ cargo test
   ...
   test result: ok. 150 passed; 0 failed; 0 ignored; 0 measured
```

## 📸 Screenshots (if applicable)

<!-- Add screenshots for UI changes -->

## 🔗 Additional Context

<!-- Add any other context about the pull request here -->

## 🚀 Deployment Notes

<!-- Any special deployment steps or considerations -->

- [ ] Database migrations need to be run
- [ ] Environment variables need to be updated
- [ ] Configuration changes are required
- [ ] Requires restart of services
- [ ] Dependencies need to be updated

---

### Reviewer Checklist

For reviewers - please verify:

- [ ] Code follows project conventions and style
- [ ] Tests are adequate and pass
- [ ] Documentation is complete and accurate
- [ ] Security considerations are addressed
- [ ] Performance implications are acceptable
- [ ] Breaking changes are clearly documented
- [ ] EventBus consistency is maintained (TransactionalEventBus usage)
- [ ] Event validation is implemented where needed
- [ ] Tenant validation is implemented where needed
