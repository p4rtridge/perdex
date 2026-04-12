import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
    title: "Perdex",
    description: "A decentralized comic platform",
    srcDir: 'docs',
    vite: {
        optimizeDeps: {
            include: ['mermaid'],
        },
    },
    markdown: {
        theme: {
            light: 'github-light-default',
            dark: 'github-dark-dimmed'
        },
        config(md) {
            // Transform ```mermaid blocks into <Mermaid> Vue components.
            // Code is URL-encoded so multi-line content is safe as an HTML attribute.
            const defaultFence = md.renderer.rules.fence ?? function (tokens, idx, options, _env, self) {
                return self.renderToken(tokens, idx, options)
            }
            md.renderer.rules.fence = function (tokens, idx, options, env, self) {
                const token = tokens[idx]
                if (token.info.trim().startsWith('mermaid')) {
                    const encoded = encodeURIComponent(token.content)
                    const html = defaultFence(tokens, idx, options, env, self)
                    return html.replace(/<pre[\s\S]*?<\/pre>/, `<Mermaid code="${encoded}" />`)
                }
                return defaultFence(tokens, idx, options, env, self)
            }
        }
    },
    themeConfig: {
        nav: [
            { text: 'Home', link: '/' },
            { text: 'ActivityPub', link: '/activitypub' },
        ],
        sidebar: [
            {
                text: 'Documentation',
                items: [
                    { text: 'ActivityPub', link: '/activitypub' },
                ]
            }
        ],

        socialLinks: [
            { icon: 'github', link: 'https://github.com/perdex-org/documentation' }
        ]
    }
})