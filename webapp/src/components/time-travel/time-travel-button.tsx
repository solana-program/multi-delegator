import { useState, useEffect, useCallback } from 'react'
import { Clock, RotateCcw } from 'lucide-react'
import { useWalletUi } from '@wallet-ui/react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger,
} from '@/components/ui/dialog'
import { useTimeTravel } from '@/hooks/use-time-travel'
import { useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

const QUICK_JUMPS = [
  { label: '+1h', seconds: 3600 },
  { label: '+6h', seconds: 21600 },
  { label: '+1d', seconds: 86400 },
  { label: '+7d', seconds: 604800 },
  { label: '+30d', seconds: 2592000 },
] as const

const DRIFT_THRESHOLD_SEC = 30

export function TimeTravelButton() {
  const { cluster } = useWalletUi()
  const isLocalnet = cluster.id === 'solana:localnet'

  if (!isLocalnet) return null

  return <TimeTravelDialog />
}

function TimeTravelDialog() {
  const { timeTravel, getCurrentTimestamp } = useTimeTravel()
  const queryClient = useQueryClient()
  const [open, setOpen] = useState(false)
  const [currentTime, setCurrentTime] = useState<number | null>(null)
  const [loading, setLoading] = useState(false)
  const [date, setDate] = useState('')
  const [hour, setHour] = useState('12')
  const [timeTraveled, setTimeTraveled] = useState(false)

  const updateDrift = useCallback((blockTime: number) => {
    const wallTime = Math.floor(Date.now() / 1000)
    setTimeTraveled(Math.abs(blockTime - wallTime) > DRIFT_THRESHOLD_SEC)
  }, [])

  const fetchTime = useCallback(async () => {
    try {
      const ts = await getCurrentTimestamp()
      setCurrentTime(ts)
      updateDrift(ts)
    } catch (e) {
      console.warn('[TimeTravel] Failed to fetch clock:', e)
    }
  }, [getCurrentTimestamp, updateDrift])

  useEffect(() => { fetchTime() }, [fetchTime])

  useEffect(() => {
    if (open) fetchTime()
  }, [open, fetchTime])

  const handleQuickJump = async (seconds: number) => {
    if (!currentTime) return
    setLoading(true)
    try {
      await timeTravel(currentTime + seconds)
      await fetchTime()
      queryClient.invalidateQueries()
      setTimeout(() => queryClient.invalidateQueries(), 500)
      toast.success('Clock advanced')
    } catch (e) {
      toast.error(e instanceof Error ? e.message : 'Time travel failed')
    } finally {
      setLoading(false)
    }
  }

  const handleCustomJump = async () => {
    if (!date || !currentTime) return
    setLoading(true)
    try {
      const ts = Math.floor(new Date(`${date}T${hour.padStart(2, '0')}:00:00`).getTime() / 1000)
      if (ts <= currentTime) {
        toast.error('Cannot travel to the past')
        setLoading(false)
        return
      }
      await timeTravel(ts)
      await fetchTime()
      queryClient.invalidateQueries()
      setTimeout(() => queryClient.invalidateQueries(), 500)
      toast.success('Clock set')
    } catch (e) {
      toast.error(e instanceof Error ? e.message : 'Time travel failed')
    } finally {
      setLoading(false)
    }
  }


  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button
          variant="outline"
          size="icon"
          className={timeTraveled
            ? 'relative ring-2 ring-green-500/50 shadow-[0_0_12px_rgba(34,197,94,0.4)] animate-[glow_2s_ease-in-out_infinite]'
            : 'relative'}
          title="Time Travel (Dev)"
        >
          <Clock className={timeTraveled ? 'h-4 w-4 text-green-400' : 'h-4 w-4 text-muted-foreground'} />
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-md border-emerald-500/20 bg-gradient-to-br from-emerald-950/40 via-slate-950 to-slate-950">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Clock className="h-5 w-5 text-emerald-400" />
            Time Travel
          </DialogTitle>
        </DialogHeader>

        <div className="grid gap-4">
          <div className="rounded-md bg-neutral-800 p-3 text-sm font-mono text-center">
            {currentTime !== null
              ? new Date(currentTime * 1000).toLocaleString()
              : 'Fetching...'}
          </div>

          <div className="grid gap-2">
            <Label className="text-xs font-medium uppercase tracking-wider text-emerald-400">Quick Jump</Label>
            <div className="flex gap-2 flex-wrap">
              {QUICK_JUMPS.map(({ label, seconds }) => (
                <Button
                  key={label}
                  variant="outline"
                  size="sm"
                  disabled={loading || !currentTime}
                  onClick={() => handleQuickJump(seconds)}
                >
                  {label}
                </Button>
              ))}
            </div>
          </div>

          <div className="h-px bg-border" />

          <div className="grid gap-2">
            <Label className="text-xs font-medium uppercase tracking-wider text-emerald-400">Jump to Date</Label>
            <div className="flex gap-2">
              <Input
                type="date"
                value={date}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => setDate(e.target.value)}
                min={currentTime ? new Date(currentTime * 1000).toLocaleDateString('en-CA') : undefined}
                className="flex-1"
              />
              <select
                value={hour}
                onChange={(e) => setHour(e.target.value)}
                className="h-10 rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
              >
                {Array.from({ length: 24 }, (_, i) => (
                  <option key={i} value={i.toString()}>
                    {i.toString().padStart(2, '0')}:00
                  </option>
                ))}
              </select>
            </div>
            <Button
              variant="outline"
              disabled={loading || !date}
              onClick={handleCustomJump}
              className="w-full font-bold text-base h-11 rounded-full border-green-500/30 bg-green-600 hover:bg-green-500 text-white shadow-[0_0_12px_rgba(34,197,94,0.3)]"
            >
              Jump
            </Button>
          </div>

          {timeTraveled && (<>
            <div className="h-px bg-border" />
            <div className="flex items-start gap-2 rounded-md border border-amber-500/30 bg-amber-500/5 p-3 text-sm text-amber-300">
              <RotateCcw className="h-4 w-4 mt-0.5 shrink-0" />
              <span>Clock is ahead of system time. Run <code className="px-1 py-0.5 rounded bg-neutral-800 text-xs">just webapp-clean</code> and restart to reset.</span>
            </div>
          </>)}
        </div>
      </DialogContent>
    </Dialog>
  )
}
