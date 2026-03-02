import { useState, useCallback, useRef } from 'react'
import { api } from '@/lib/api-client'

export interface SetupStep {
  id: string
  label: string
  status: 'pending' | 'running' | 'done' | 'error'
  message?: string
}

const POLL_INTERVAL = 2000

export function useLocalnetSetup() {
  const [steps, setSteps] = useState<SetupStep[]>([
    { id: 'start-validator', label: 'Start Surfpool validator', status: 'pending' },
    { id: 'wait-validator', label: 'Wait for validator', status: 'pending' },
    { id: 'wait-program', label: 'Wait for program deployment', status: 'pending' },
    { id: 'create-usdc', label: 'Create mock USDC', status: 'pending' },
  ])
  const [isComplete, setIsComplete] = useState(false)
  const [result, setResult] = useState<{ programId: string; usdcMint: string } | null>(null)
  const abortRef = useRef(false)

  const updateStep = useCallback((id: string, update: Partial<SetupStep>) => {
    setSteps(prev => prev.map(s => s.id === id ? { ...s, ...update } : s))
  }, [])

  const poll = useCallback(async (
    check: () => Promise<boolean>,
    maxAttempts = 60,
  ): Promise<void> => {
    for (let i = 0; i < maxAttempts; i++) {
      if (abortRef.current) throw new Error('Setup cancelled')
      if (await check()) return
      await new Promise(r => setTimeout(r, POLL_INTERVAL))
    }
    throw new Error('Timed out')
  }, [])

  const run = useCallback(async () => {
    abortRef.current = false

    try {
      updateStep('start-validator', { status: 'running', message: 'Starting surfpool...' })
      await api.setup.startValidator()
      updateStep('start-validator', { status: 'done', message: 'Surfpool started' })

      updateStep('wait-validator', { status: 'running', message: 'Waiting for RPC...' })
      await poll(async () => {
        const s = await api.setup.validatorStatus()
        return s.validatorRunning
      })
      updateStep('wait-validator', { status: 'done', message: 'Validator ready' })

      updateStep('wait-program', { status: 'running', message: 'Waiting for program...' })
      let programAddress = ''
      await poll(async () => {
        const s = await api.setup.validatorStatus()
        if (s.programDeployed) programAddress = s.programAddress
        return s.programDeployed
      })
      updateStep('wait-program', { status: 'done', message: 'Program deployed' })

      updateStep('create-usdc', { status: 'running', message: 'Creating mock USDC...' })
      const usdcResult = await api.setup.createMockUsdc()
      updateStep('create-usdc', {
        status: 'done',
        message: usdcResult.alreadyExisted ? 'USDC already exists' : 'USDC created',
      })

      setResult({
        programId: programAddress,
        usdcMint: usdcResult.mint,
      })
      setIsComplete(true)
    } catch (error) {
      const msg = error instanceof Error ? error.message : 'Unknown error'
      setSteps(prev => {
        const running = prev.find(s => s.status === 'running')
        if (!running) return prev
        return prev.map(s => s.id === running.id ? { ...s, status: 'error' as const, message: msg } : s)
      })
    }
  }, [updateStep, poll])

  return { steps, run, isComplete, result }
}

export function useDevnetSetup() {
  const [steps, setSteps] = useState<SetupStep[]>([
    { id: 'connect-wallet', label: 'Connect wallet', status: 'pending' },
    { id: 'deploy-program', label: 'Deploy program', status: 'pending' },
    { id: 'create-usdc', label: 'Create mock USDC', status: 'pending' },
    { id: 'mint-usdc', label: 'Mint USDC to wallet', status: 'pending' },
    { id: 'save-config', label: 'Save configuration', status: 'pending' },
  ])
  const [isComplete, setIsComplete] = useState(false)
  const [result, setResult] = useState<{ programId: string; usdcMint: string } | null>(null)

  const updateStep = useCallback((id: string, update: Partial<SetupStep>) => {
    setSteps(prev => prev.map(s => s.id === id ? { ...s, ...update } : s))
  }, [])

  const markStepDone = useCallback((id: string, message: string) => {
    updateStep(id, { status: 'done', message })
  }, [updateStep])

  const markStepError = useCallback((id: string, message: string) => {
    updateStep(id, { status: 'error', message })
  }, [updateStep])

  const markStepRunning = useCallback((id: string, message: string) => {
    updateStep(id, { status: 'running', message })
  }, [updateStep])

  const completeSetup = useCallback((programId: string, usdcMint: string) => {
    setResult({ programId, usdcMint })
    setIsComplete(true)
  }, [])

  return {
    steps,
    isComplete,
    result,
    updateStep,
    markStepDone,
    markStepError,
    markStepRunning,
    completeSetup,
    setResult,
  }
}
