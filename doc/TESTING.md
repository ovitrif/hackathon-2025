# Test Suite

## Create a Staging user

Get a invite code on Staging:

```
curl -X GET \
"https://admin.homeserver.staging.pubky.app/generate_signup_token" \
  -H "X-Admin-Password: voyage tuition cabin arm stock guitar soon salute"
```

On Pubky Ring, register an account

- with this invite code
- with the Staging Homeserver PK: ufibwbmed6jeq9k4p583go95wofakh9fwpp4k734trq79pd9u1uy

## Custom Markdown Wiki Page Links

### Link Format
```
(Display Text)[userid/pageid]
```

Example:
```markdown
Check out (Alice's Page)[alice_user_id/550e8400-e29b-41d4-a716-446655440000]
```

### Usage
- After authentication, click "Open Wiki" button to access the wiki
- Links automatically navigate between pages in the same view
- Links are intercepted and handled internally (won't open in browser)

### Page Storage
- Currently: In-memory `PageStore` with test pages (home, Alice, Bob)
- Future: Ready to swap with pubky link-based storage
- Page format: `HashMap<String, String>` mapping page IDs to markdown content

### Logging
Run with `RUST_LOG=info cargo run` to see:
- Link clicks: `🔗 Intercepted link click: <url>`
- Navigation: `📄 Navigating to: <url>`
- Page transitions: `Navigate to page: <from> -> <to>`
