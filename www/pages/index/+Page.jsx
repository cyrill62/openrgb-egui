import './style.css'
import logo from '../../assets/logo.svg'
import { MotionConfig, motion } from 'motion/react'

const github = 'https://github.com/cyrill62/pimpbox-rgb'
const colors = ['#ff4b91', '#bc65ff', '#6486ff', '#41d9d0', '#a0f078', '#ffd166']
const screenshots = [
  ['00-overview.png', 'Device overview'],
  ['02-keyboard-leds.png', 'Paint individual keyboard LEDs'],
  ['05-motherboard-zones.png', 'Fill and resize device zones'],
  ['08-profiles.png', 'Save and load lighting profiles'],
].map(([file, title]) => ({ src: `${import.meta.env.BASE_URL}${file}`, title }))

export default function Page() {
  return (
    <MotionConfig reducedMotion="user">
    <main data-theme="night" className="site-shell min-h-screen bg-base-100 text-base-content">
      <header className="navbar mx-auto max-w-6xl px-5 sm:px-8">
        <a className="flex items-center gap-3 font-bold tracking-wide" href="#top">
          <span className="brand-mark" aria-hidden="true">PB</span>
          <span>PimpBox RGB</span>
        </a>
        <nav className="ml-auto flex items-center gap-2 sm:gap-4" aria-label="Main navigation">
          <a className="btn btn-ghost btn-sm" href="#features">Features</a>
          <a className="btn btn-primary btn-sm" href={`${github}/releases/latest`}>Download</a>
        </nav>
      </header>

      <section id="top" className="hero min-h-[72vh] px-5 py-20">
        <div className="hero-content grid max-w-6xl gap-16 lg:grid-cols-[1.15fr_.85fr]">
          <motion.div initial={{ opacity: 0, y: 22 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.65, ease: 'easeOut' }}>
            <div className="badge badge-outline mb-6 border-primary/50 px-4 py-3 text-primary">OpenRGB, with attitude</div>
            <h1 className="max-w-[680px]"><img className="block w-full" src={logo} alt="PimpBox RGB — Pimp My Box: RGB Edition" /></h1>
            <p className="mt-7 max-w-xl text-lg leading-8 text-base-content/70">
              Take control of your lighting. Browse devices, pick a mode, paint individual LEDs, resize zones, and manage profiles.
            </p>
            <div className="mt-9 flex flex-wrap gap-3">
              <motion.a className="btn btn-primary btn-lg" href={`${github}/releases/latest`} whileHover={{ y: -2 }} whileTap={{ scale: 0.97 }}>Get the latest release <span aria-hidden="true">↗</span></motion.a>
              <motion.a className="btn btn-outline btn-lg" href={github} whileHover={{ y: -2 }} whileTap={{ scale: 0.97 }}>View source</motion.a>
            </div>
            <p className="mt-5 text-sm text-base-content/50">Requires a running OpenRGB SDK server.</p>
          </motion.div>

          <motion.div className="rgb-panel card border border-white/10 bg-base-200/70 shadow-2xl" aria-label="Decorative RGB lighting preview" initial={{ opacity: 0, scale: 0.96 }} animate={{ opacity: 1, scale: 1 }} transition={{ duration: 0.7, delay: 0.15, ease: 'easeOut' }}>
            <div className="card-body gap-5 p-6 sm:p-8">
              <div className="flex items-center justify-between">
                <div><div className="text-xs uppercase tracking-[.2em] text-base-content/45">Lighting preview</div><div className="mt-1 font-semibold">Your setup, your colors</div></div>
                <span className="status status-success" aria-label="Connected" />
              </div>
              <div className="device-frame">
                <div className="device-top"><span>MAINBOARD</span><span>ARGB ZONE 1</span></div>
                <div className="led-grid" aria-hidden="true">
                  {colors.concat(colors, colors).map((color, i) => <i key={i} style={{ '--led': color, '--delay': `${i * 80}ms` }} />)}
                </div>
                <div className="device-bottom"><span>OpenRGB SDK</span><span>12 LEDs</span></div>
              </div>
              <div className="flex flex-wrap gap-2">
                {['Modes', 'LED maps', 'Zones', 'Profiles'].map((label) => <span className="badge badge-neutral" key={label}>{label}</span>)}
              </div>
            </div>
          </motion.div>
        </div>
      </section>

      <section id="features" className="mx-auto max-w-6xl px-5 py-16 sm:px-8">
        <div className="mb-8 max-w-2xl">
          <p className="font-semibold uppercase tracking-[.2em] text-primary">Pick your interface</p>
          <h2 className="mt-3 text-3xl font-bold tracking-tight sm:text-4xl">One project. Two ways to use it.</h2>
        </div>
        <div className="grid gap-4 md:grid-cols-3">
          <motion.article className="card feature-card" initial={{ opacity: 0, y: 20 }} whileInView={{ opacity: 1, y: 0 }} whileHover={{ y: -4 }} viewport={{ once: true, amount: 0.2 }} transition={{ duration: 0.45 }}><div className="card-body"><div className="feature-icon">▦</div><h3 className="card-title">Desktop app</h3><p className="text-base-content/65">Live device previews, visual LED maps, color painting, hardware modes, zones, and profiles.</p></div></motion.article>
          <motion.article className="card feature-card" initial={{ opacity: 0, y: 20 }} whileInView={{ opacity: 1, y: 0 }} whileHover={{ y: -4 }} viewport={{ once: true, amount: 0.2 }} transition={{ duration: 0.45, delay: 0.1 }}><div className="card-body"><div className="feature-icon">⌘</div><h3 className="card-title">Terminal UI</h3><p className="text-base-content/65">A keyboard-friendly controller for headless machines and SSH sessions, with mouse support too.</p></div></motion.article>
          <motion.article className="card feature-card" initial={{ opacity: 0, y: 20 }} whileInView={{ opacity: 1, y: 0 }} whileHover={{ y: -4 }} viewport={{ once: true, amount: 0.2 }} transition={{ duration: 0.45, delay: 0.2 }}><div className="card-body"><div className="feature-icon">✳</div><h3 className="card-title">OpenRGB powered</h3><p className="text-base-content/65">Connects to the OpenRGB SDK server and controls hardware OpenRGB has already detected.</p></div></motion.article>
        </div>
      </section>

      <section className="mx-auto max-w-6xl px-5 py-10 sm:px-8">
        <div className="mb-8 max-w-2xl">
          <p className="font-semibold uppercase tracking-[.2em] text-primary">Take a look</p>
          <h2 className="mt-3 text-3xl font-bold tracking-tight sm:text-4xl">See PimpBox RGB in action.</h2>
        </div>
        <div className="grid gap-5 sm:grid-cols-2">
          {screenshots.map(({ src, title }, index) => (
            <motion.figure className="card overflow-hidden border border-white/10 bg-base-200/70" key={src} initial={{ opacity: 0, y: 22 }} whileInView={{ opacity: 1, y: 0 }} viewport={{ once: true, amount: 0.12 }} transition={{ duration: 0.5, delay: index * 0.08 }}>
              <a href={src} target="_blank" rel="noreferrer" aria-label={`Open screenshot: ${title}`}>
                <motion.img className="screenshot-image w-full" src={src} alt={title} loading="lazy" whileHover={{ scale: 1.02 }} transition={{ duration: 0.25 }} />
              </a>
              <figcaption className="px-5 py-4 font-medium">{title}</figcaption>
            </motion.figure>
          ))}
        </div>
      </section>

      <section className="mx-auto max-w-6xl px-5 pb-24 pt-10 sm:px-8">
        <motion.div className="card border border-primary/20 bg-gradient-to-br from-primary/10 via-base-200 to-base-200" initial={{ opacity: 0, y: 20 }} whileInView={{ opacity: 1, y: 0 }} viewport={{ once: true, amount: 0.2 }} transition={{ duration: 0.5 }}>
          <div className="card-body gap-5 p-7 sm:p-10 md:flex-row md:items-center md:justify-between">
            <div><h2 className="text-2xl font-bold">Ready to light things up?</h2><p className="mt-2 text-base-content/65">Start the OpenRGB SDK server, then grab a release with both apps included.</p></div>
            <a className="btn btn-primary" href={`${github}/releases/latest`}>Download PimpBox RGB <span aria-hidden="true">↗</span></a>
          </div>
        </motion.div>
      </section>

      <footer className="border-t border-white/10">
        <div className="mx-auto flex max-w-6xl flex-col gap-3 px-5 py-8 text-sm text-base-content/55 sm:flex-row sm:items-center sm:justify-between sm:px-8">
          <span>PimpBox RGB · Pimp My Box: RGB Edition</span>
          <div className="flex gap-5"><a className="link link-hover" href="https://openrgb.org">OpenRGB</a><a className="link link-hover" href={github}>GitHub</a></div>
        </div>
      </footer>
    </main>
    </MotionConfig>
  )
}
