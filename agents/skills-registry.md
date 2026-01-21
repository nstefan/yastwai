# Skills Registry

This file maps project types and technologies to recommended skills from [skills.sh](https://skills.sh).

## How Agents Use This File

1. **Detect** project technologies by examining files (package.json, Cargo.toml, etc.)
2. **Match** detected technologies to skill categories below
3. **Suggest** relevant skills that aren't already installed
4. **Install** with user approval: `npx skills add <owner/repo>`

## Installed Skills

Track installed skills here to avoid duplicate suggestions:

```
<!-- Add installed skills here -->
<!-- Example: vercel-labs/agent-skills (react-best-practices) -->
```

---

## Detection Rules

### File-Based Detection

| If You Find | Technology | Suggest Category |
|-------------|------------|------------------|
| `package.json` with "react" | React | react, web-design |
| `package.json` with "next" | Next.js | react, nextjs |
| `package.json` with "vue" | Vue | vue |
| `package.json` with "nuxt" | Nuxt | nuxt |
| `package.json` with "expo" | Expo/React Native | expo, mobile |
| `app.json` with "expo" | Expo | expo, mobile |
| `Cargo.toml` | Rust | rust |
| `go.mod` | Go | go |
| `Package.swift` or `*.xcodeproj` | Swift/iOS | swift, ios |
| `pyproject.toml` or `requirements.txt` | Python | python |
| `tailwind.config.*` | Tailwind CSS | tailwind, design |
| `Dockerfile` | Docker | devops |
| `.github/workflows/*` | GitHub Actions | ci-cd |

### Dependency-Based Detection

When parsing `package.json` dependencies:

| Dependency | Suggest |
|------------|---------|
| `better-auth` | better-auth-best-practices |
| `remotion` | remotion-best-practices |
| `shadcn` or `@radix-ui/*` | shadcn-ui |
| `tailwindcss` | tailwind patterns |
| `@tanstack/react-query` | data-fetching patterns |

---

## Skills by Category

### React & Web

| Skill | Repo | Install Command | When to Suggest |
|-------|------|-----------------|-----------------|
| React Best Practices | `vercel-labs/agent-skills` | `npx skills add vercel-labs/agent-skills` | React projects |
| Web Design Guidelines | `vercel-labs/agent-skills` | `npx skills add vercel-labs/agent-skills` | Any web frontend |

### Mobile (Expo / React Native)

| Skill | Repo | Install Command | When to Suggest |
|-------|------|-----------------|-----------------|
| Upgrading Expo | `expo/skills` | `npx skills add expo/skills` | Expo projects |
| Building UI | `expo/skills` | `npx skills add expo/skills` | Expo projects |
| Data Fetching | `expo/skills` | `npx skills add expo/skills` | Expo projects |
| Dev Client | `expo/skills` | `npx skills add expo/skills` | Expo projects |
| Use DOM | `expo/skills` | `npx skills add expo/skills` | Expo web bridges |

### Video & Media

| Skill | Repo | Install Command | When to Suggest |
|-------|------|-----------------|-----------------|
| Remotion Best Practices | `remotion-dev/skills` | `npx skills add remotion-dev/skills` | Remotion projects |

### Authentication

| Skill | Repo | Install Command | When to Suggest |
|-------|------|-----------------|-----------------|
| Better Auth Best Practices | `better-auth/skills` | `npx skills add better-auth/skills` | Using better-auth |

### Security

| Skill | Repo | Install Command | When to Suggest |
|-------|------|-----------------|-----------------|
| Vulnerability Scanning | `trailofbits/skills` | `npx skills add trailofbits/skills` | Security audits |
| Code Security | `trailofbits/skills` | `npx skills add trailofbits/skills` | Security reviews |

### AI/Agent Development

| Skill | Repo | Install Command | When to Suggest |
|-------|------|-----------------|-----------------|
| Skill Creator | `anthropics/skills` | `npx skills add anthropics/skills` | Creating new skills |
| PDF Manipulation | `anthropics/skills` | `npx skills add anthropics/skills` | PDF workflows |

### DevOps & Infrastructure

| Skill | Repo | Install Command | When to Suggest |
|-------|------|-----------------|-----------------|
| Vercel Deploy | `vercel-labs/agent-skills` | `npx skills add vercel-labs/agent-skills` | Vercel deployments |
| Cloudflare Workers | `cloudflare/skills` | `npx skills add cloudflare/skills` | Cloudflare projects |

---

## Suggestion Prompt Template

When suggesting skills, agents should use this format:

```
Based on your project, I recommend installing these skills:

**[Skill Name]** - [Brief description]
Install: `npx skills add [repo]`

Would you like me to install any of these skills?
```

---

## Adding New Skills

1. Find skills at https://skills.sh
2. Add to appropriate category table above
3. Add detection rules if needed
4. Keep this registry updated as you discover useful skills

## Notes

- Skills are installed project-wide and enhance agent capabilities
- Multiple skills from the same repo are installed together
- Check skills.sh leaderboard for popular/trending skills
- Prefer well-maintained skills with high install counts
