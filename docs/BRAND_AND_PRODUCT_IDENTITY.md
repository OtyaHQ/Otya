# Otya brand and product identity system

**Status:** proposed brand system for product and UI implementation; not a final approved logo artwork or trademark/legal clearance.  
**Reviewed:** 2026-11-28.

## 1. Brand position

Otya is an independent software platform that people install and operate themselves. It provides the engine, interfaces, identity foundations, reusable services and tools to build and operate applications on infrastructure chosen by the operator.

The brand must communicate ownership and independence, not vendor lock-in; engineering capability without making the product feel developer-only; trust and control without claiming to host or secure every external service; openness, interoperability and provider choice; and one coherent system with modular capabilities.

### Recommended brand promise

**Your platform. Your infrastructure. Your control.**

Supporting statement:

**Build any application on Otya. Run it on infrastructure you choose. Keep ownership of your accounts, data, credentials and deployments.**

Alternative shorter product line: **Build freely. Operate independently.**

Do not market Otya as "the cloud", "everything hosted for you", or an automatic replacement for Cloudflare, Firebase, Supabase or other providers. Otya connects and manages services that the operator owns.

## 2. Brand architecture

Use one master brand, with clear product names beneath it. Do not create many unrelated logos before the capabilities are established.

| Name | Role | Branding rule |
|---|---|---|
| Otya | Master brand and ecosystem identity | Primary wordmark and visual identity |
| Otya Engine | Portable core, runtime and platform distribution | Main product name; this is the installable platform |
| Otya Console | Visual operator/admin UI | "Otya" master brand with Console descriptor |
| Otya Identity | Installation-owned login, user/group and OIDC service | Each installation operates its own identity realm |
| Otya Connect | Provider account connections, API credentials, webhooks and adapters | Provider integrations remain visibly third-party |
| Otya Studio | Optional visual app builder, UI kit and workflow authoring environment | A capability of the platform; never imply every app can be created without code |
| Otya Modules | First-party reusable platform capabilities | Individual modules use the Otya mark plus a plain functional label |

Names such as Console, Identity, Connect and Studio are proposed suite labels. They can initially appear as navigation sections within one Otya Engine product rather than separate legal products or standalone services.

Keep customer/company branding separate. Every independent Engine installation can set its own organization name, logo, colours, domain, email sender details and portal text while the platform itself retains Otya attribution and version information where configured.

## 3. Visual direction

### Logo concept

Develop an original, compact geometric O mark based on an open loop/path with three connected points. It should suggest an engine/core at the centre, modular capabilities connected through stable interfaces, and independent paths that remain under the operator's control.

Pair the mark with a distinctive "Otya" wordmark. The mark must remain legible as a 16–24 px favicon, monochrome print symbol, terminal icon and app icon.

Avoid copying or closely resembling the Cloudflare orange cloud, Firebase flame, Supabase twin bars, Kubernetes wheel or another infrastructure brand. Do not make a generic cloud the central logo: Otya does not host the operator's infrastructure.

The final master logo requires custom vector artwork, monochrome and reversed versions, clear-space rules, minimum sizes, favicon variants and trademark/design review. This specification is not itself the final logo.

### Provisional core palette

Use a calm, technical base with a distinctive electric indigo/blue accent and a restrained teal for successful connectivity. This palette is a proposal pending contrast and user-interface testing.

| Token | Proposed value | Use |
|---|---|---|
| Otya Ink | #101828 | Dark surfaces, primary text |
| Otya Indigo | #4355DB | Primary action and brand anchor |
| Otya Blue | #2875E8 | Links, focus and active states |
| Otya Teal | #159A91 | Connected/healthy state, sparingly |
| Otya Amber | #D99524 | Attention and warning |
| Otya Red | #C44747 | Errors and destructive actions |
| Canvas Light | #F6F8FC | Light app background |
| Surface Light | #FFFFFF | Cards and panels |
| Canvas Dark | #0D1424 | Dark app background |
| Surface Dark | #151F33 | Dark panels |

Status colours must not be used as decorative brand colours. Never rely on colour alone to convey success, failure, permission or danger. Verify text, icon, focus and control contrast in both themes against WCAG 2.2 AA targets before release. Provide high-contrast mode and respect reduced-motion preferences.

### Typography

Use a modern, highly legible sans-serif such as Inter or an equivalent open font for interface text. Use a clear monospace face for code, identifiers, logs and technical values. Do not ship unlicensed fonts or depend on remote font downloads for a private/offline installation.

### Shape, icon and motion

- Use simple geometry, consistent 8 px spacing rhythm and restrained corner radii.
- Use a consistent line icon family with labels for unfamiliar actions.
- Avoid excessive gradients, glass effects, animated backgrounds and decorative charts.
- Motion should explain navigation, progress and state change; support reduced motion and never gate access to information on animation.
- Use charts only when they expose a real operational measure and its time range.

## 4. Product UI principles

Otya must serve developers and non-developers. The Console should translate infrastructure details into understandable tasks without hiding the real consequences.

- Lead with what the operator wants to accomplish, then disclose the underlying provider/resource.
- Explain whether a capability is local, Otya-provided software, or an external provider the operator must supply.
- Show account owner, provider, resource, environment, granted scope, last verification and billing owner.
- Explain missing configuration in plain language and provide a guided next step.
- Show a preview before deployment, credential changes, migrations, DNS changes or destructive operations.
- Never show a green "connected" state if only a token is saved but the required resource or capability is absent.
- Provide a clear distinction among "not configured", "credentials invalid", "permission missing", "provider unavailable", "resource missing", "healthy" and "degraded".
- Make advanced controls available without forcing them on ordinary business users.
- Keep changes exportable to versioned, reviewable configuration and API calls; visual configuration must not create hidden proprietary state.

## 5. Voice and writing

Tone: direct, capable, calm, clear and transparent.

Use "Connect your Cloudflare account" rather than "Otya provisions your cloud"; "Your provider account will be billed by the provider" rather than an ambiguous claim of free hosting; "Credential saved in this installation's vault" rather than "Otya stores your password"; and "Connection validated; R2 bucket permission missing" rather than a vague red error.

Avoid exaggerated promises such as "unlimited", "zero maintenance", "unhackable" or "everything automatically"; provider impersonation or hiding external billing; calling a provider's service an Otya-hosted service; describing unimplemented plans as available features.

## 6. Naming and interface language

Prefer plain labels: Projects, Components, Environments, Connections, Resources, Credentials, Webhooks, Permissions, Deployments, Backups and Activity.

Use "Provider connection" for an account/API credential relationship; "Resource binding" for a specific database, bucket, Worker or project; "Secret" only for a sensitive value; and "Webhook endpoint" for inbound event delivery.

Do not use "tenant" in normal UI copy. The product model is one operator installation with workspaces and projects, not a hosted multi-tenant account hierarchy.

## 7. White-label and rebranding boundaries

Every installation can customise organization display name and logo; login portal and email sender branding; console theme and accent colour; custom domain and links; application-specific logos and themes; notification templates and support contact details.

Customisation must not silently remove ownership/security attribution, provider disclosures or software version/security notices. Keep Otya identity configurable according to the future license and trademark policy. No license or white-label right is implied by this design document.

## 8. Deliverables before brand release

- Approved master logo in vector and raster variants.
- Wordmark, icon-only, monochrome, reversed and favicon assets.
- Brand colour tokens and accessible light/dark theme.
- UI component library with design tokens.
- Console, login portal, Connect catalogue and Studio interaction patterns.
- Product naming/trademark and domain review.
- Voice and writing guide.
- Brand governance, contribution and asset licensing rules.

## 9. Current status

The name hierarchy, promise, visual direction, palette, UI principles and white-label boundaries are proposed. No final logo artwork, theme package, font license, trademark clearance or deployed branded console has been produced. Do not represent these as completed deliverables.
