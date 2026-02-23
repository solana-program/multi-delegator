import { useState } from 'react'
import { ClipboardPen, Plus } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { PlanCard } from './plan-card'
import { CreatePlanDialog } from './create-plan-dialog'
import { useMyPlans } from '@/hooks/use-plans'

export function MyPlansPanel() {
  const [dialogOpen, setDialogOpen] = useState(false)
  const { data: plans, isLoading } = useMyPlans()

  if (isLoading) {
    return (
      <Card className="border-emerald-500/20 bg-gradient-to-br from-emerald-950/40 via-emerald-900/20 to-transparent">
        <CardContent className="flex items-center justify-center py-12">
          <div className="animate-pulse text-muted-foreground">Loading plans...</div>
        </CardContent>
      </Card>
    )
  }

  const hasPlan = plans && plans.length > 0

  return (
    <Card className="relative overflow-hidden border-emerald-500/20 bg-gradient-to-br from-emerald-950/40 via-emerald-900/20 to-transparent hover:border-emerald-500/40 transition-all duration-300">
      <CardHeader className="pb-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <ClipboardPen className="h-5 w-5 text-emerald-400" />
            <CardTitle>My Plans</CardTitle>
          </div>
          {hasPlan && (
            <Badge variant="outline" className="text-emerald-400 border-emerald-400/30">
              {plans.length}
            </Badge>
          )}
        </div>
      </CardHeader>
      <CardContent>
          {hasPlan ? (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {plans.map((plan) => (
                <PlanCard key={plan.address} plan={plan} />
              ))}
            </div>
          ) : (
            <div className="flex flex-col items-center justify-center py-8 gap-2 text-muted-foreground">
              <ClipboardPen className="h-8 w-8" />
              <p className="text-sm">No plans yet</p>
            </div>
          )}
          <div className="flex justify-center pt-2">
            <Button
              className="gap-2 rounded-full px-6 h-12 bg-emerald-600 hover:bg-emerald-500 text-white shadow-lg shadow-emerald-500/20 transition-all hover:shadow-emerald-500/30"
              onClick={() => setDialogOpen(true)}
            >
              <Plus className="h-5 w-5" />
              Create Plan
            </Button>
          </div>
      </CardContent>
      <CreatePlanDialog open={dialogOpen} onOpenChange={setDialogOpen} />
    </Card>
  )
}
