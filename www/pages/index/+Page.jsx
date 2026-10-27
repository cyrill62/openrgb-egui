import './style.css'
import { useEffect, useRef, useState } from 'react'
import banner from '../../assets/banner.svg'
import logo from '../../assets/logo.svg'
import { AnimatePresence, MotionConfig, motion } from 'motion/react'

const github = 'https://github.com/cyrill62/pimpbox-rgb'
const screenshots = [
  ['00-overview.png', 'Overview', 'All connected devices'],
  ['02-keyboard-leds.png', 'Keyboard', 'Paint individual LEDs'],
  ['05-motherboard-zones.png', 'Zones', 'Resize and fill device zones'],
  ['08-profiles.png', 'Profiles', 'Save and load lighting profiles'],
].map(([file, tab, title]) => ({ src: `${import.meta.env.BASE_URL}${file}`, tab, title }))

export default function Page() {
  const [activeScreenshot, setActiveScreenshot] = useState(0)
  const [modalOpen, setModalOpen] = useState(false)
  const dialogRef = useRef(null)
  const screenshot = screenshots[activeScreenshot]
  const changeScreenshot = (step) => {
    setActiveScreenshot((index) => (index + step + screenshots.length) % screenshots.length)
  }

  useEffect(() => {
    const dialog = dialogRef.current
    if (!dialog) return
    if (modalOpen && !dialog.open) dialog.showModal()
    if (!modalOpen && dialog.open) dialog.close()
  }, [modalOpen])

  return (
    <MotionConfig reducedMotion="user">
    <main data-theme="night" className="site-shell min-h-screen bg-base-100 text-base-content">
      <header className="navbar mx-auto max-w-6xl px-5 sm:px-8">
        <a className="flex items-center gap-3 font-bold tracking-wide" href="#top">
          <img className="h-8 w-8 object-contain" src={logo} alt="" aria-hidden="true" />
          <span>PimpBox RGB</span>
        </a>
        <nav className="ml-auto flex items-center gap-2 sm:gap-4" aria-label="Main navigation">
          <a className="btn btn-ghost btn-sm" href="#features">Features</a>
          <a className="btn btn-primary btn-sm" href={`${github}/releases/latest`}>Download</a>
        </nav>
      </header>

      <section id="top" className="hero min-h-[72vh] px-5 py-20">
        <div className="hero-content flex w-full max-w-6xl flex-col gap-12">
          <motion.div className="w-full text-center" initial={{ opacity: 0, y: 22 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.65, ease: 'easeOut' }}>
            <div className="badge badge-outline mb-6 border-primary/50 px-4 py-3 text-primary">OpenRGB, with attitude</div>
            <h1 className="mx-auto max-w-[680px]"><img className="block w-full" src={banner} alt="PimpBox RGB — Pimp My Box: RGB Edition" /></h1>
            <p className="mx-auto mt-7 max-w-xl text-lg leading-8 text-base-content/70">
              Take control of your lighting. Browse devices, pick a mode, paint individual LEDs, resize zones, and manage profiles.
            </p>
            <div className="mt-9 flex flex-wrap justify-center gap-3">
              <motion.a className="btn btn-primary btn-lg" href={`${github}/releases/latest`} whileHover={{ y: -2 }} whileTap={{ scale: 0.97 }}>Get the latest release <span aria-hidden="true">↗</span></motion.a>
              <motion.a className="btn btn-outline btn-lg" href={github} whileHover={{ y: -2 }} whileTap={{ scale: 0.97 }}>View source</motion.a>
            </div>
            <p className="mt-5 text-sm text-base-content/50">Requires a running OpenRGB SDK server.</p>
          </motion.div>

          <motion.div className="mx-auto w-full max-w-5xl" aria-label="PimpBox RGB app screenshots" initial={{ opacity: 0, y: 16 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.7, delay: 0.15, ease: 'easeOut' }}>
            <div className="mb-4 flex flex-wrap justify-center gap-2" role="tablist" aria-label="App screenshots">
              {screenshots.map(({ tab }, index) => (
                <button
                  key={tab}
                  type="button"
                  role="tab"
                  aria-selected={index === activeScreenshot}
                  className={`screenshot-tab ${index === activeScreenshot ? 'active' : ''}`}
                  onClick={() => setActiveScreenshot(index)}
                >
                  {tab}
                </button>
              ))}
            </div>
            <div className="screenshot-window overflow-hidden rounded-xl border border-white/10 bg-base-200 shadow-2xl">
              <div className="flex items-center gap-2 border-b border-white/10 px-4 py-3">
                <span className="window-dot" /><span className="window-dot" /><span className="window-dot" />
                <span className="ml-2 truncate text-xs text-base-content/60">{screenshot.title}</span>
              </div>
              <div className="screenshot-stage aspect-[16/10] overflow-hidden bg-base-300">
                <button className="block h-full w-full cursor-zoom-in" type="button" onClick={() => setModalOpen(true)} aria-label={`Open ${screenshot.title} screenshot carousel`}>
                  <AnimatePresence mode="wait">
                    <motion.img
                      key={screenshot.src}
                      src={screenshot.src}
                      alt={`PimpBox RGB ${screenshot.title}`}
                      width="1918"
                      height="1218"
                      className="block h-full w-full object-contain"
                      initial={{ opacity: 0, scale: 0.985 }}
                      animate={{ opacity: 1, scale: 1 }}
                      exit={{ opacity: 0, scale: 0.985 }}
                      transition={{ duration: 0.22 }}
                    />
                  </AnimatePresence>
                </button>
              </div>
            </div>
          </motion.div>
        </div>
      </section>

      <dialog
        ref={dialogRef}
        className="screenshot-modal"
        aria-label="PimpBox RGB screenshot carousel"
        onClose={() => setModalOpen(false)}
        onClick={(event) => {
          if (event.target === event.currentTarget) setModalOpen(false)
        }}
        onKeyDown={(event) => {
          if (event.key === 'ArrowLeft') changeScreenshot(-1)
          if (event.key === 'ArrowRight') changeScreenshot(1)
        }}
      >
        <div className="screenshot-modal-content">
          <div className="flex items-center justify-between gap-4 border-b border-white/10 px-5 py-4">
            <div>
              <h2 className="font-semibold">{screenshot.title}</h2>
              <p className="mt-1 text-sm text-base-content/55">{activeScreenshot + 1} / {screenshots.length}</p>
            </div>
            <button className="btn btn-circle btn-ghost btn-sm" type="button" aria-label="Close screenshot carousel" onClick={() => setModalOpen(false)}>✕</button>
          </div>
          <div className="screenshot-modal-stage">
            <button className="screenshot-arrow left" type="button" aria-label="Previous screenshot" onClick={() => changeScreenshot(-1)}>‹</button>
            <AnimatePresence mode="wait">
              <motion.img
                key={screenshot.src}
                src={screenshot.src}
                alt={`PimpBox RGB ${screenshot.title}`}
                width="1918"
                height="1218"
                className="h-full w-full object-contain"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                transition={{ duration: 0.18 }}
              />
            </AnimatePresence>
            <button className="screenshot-arrow right" type="button" aria-label="Next screenshot" onClick={() => changeScreenshot(1)}>›</button>
          </div>
          <div className="flex items-center justify-between gap-4 border-t border-white/10 px-5 py-4">
            <span className="text-sm text-base-content/60">{screenshot.tab}</span>
            <div className="flex gap-2">
              <button className="btn btn-outline btn-sm" type="button" onClick={() => changeScreenshot(-1)}>Previous</button>
              <button className="btn btn-primary btn-sm" type="button" onClick={() => changeScreenshot(1)}>Next</button>
            </div>
          </div>
        </div>
      </dialog>

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
