"""
Evolución No Supervisada - Sistema de auto-mejora para Eden.

Este módulo implementa mecanismos de evolución que permiten a Eden:
- Detectar gaps en conocimiento y comportamiento
- Identificar áreas de mejora de forma autónoma
- Reforzar conexiones valiosas en el Knowledge Graph
- Adaptar parámetros según patrones de uso
- Mantener coherencia y salud del sistema

La evolución ocurre en ciclos periódicos y no requiere intervención externa.
"""

from __future__ import annotations

import asyncio
import json
import logging
import random
import uuid
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import Enum
from typing import Optional, Callable, Any
from collections import defaultdict

logger = logging.getLogger(__name__)


class EvolutionPhase(Enum):
    """Fases del ciclo de evolución."""
    INTROSPECTION = "introspection"      # Analizar estado actual
    DIAGNOSIS = "diagnosis"              # Identificar problemas
    PROPOSAL = "proposal"               # Generar hipótesis de mejora
    TRIAL = "trial"                     # Probar cambios en espacio seguro
    ADOPTION = "adoption"               # Aplicar cambios aprobados
    VERIFICATION = "verification"       # Validar que los cambios funcionan


class GapType(Enum):
    """Tipos de gaps detectados."""
    KNOWLEDGE = "knowledge"             # Falta de conocimiento en dominio
    CONNECTION = "connection"           # Relaciones faltantes en KG
    BEHAVIOR = "behavior"               # Patrones de respuesta mejorables
    COHERENCE = "coherence"             # Inconsistencias en identidad
    STALENESS = "staleness"             # Conocimiento desactualizado


@dataclass
class EvolutionGap:
    """Representa un gap identificado en el sistema."""
    id: str
    gap_type: GapType
    description: str
    severity: float  # 0.0 - 1.0
    evidence: list[str] = field(default_factory=list)
    suggested_fixes: list[str] = field(default_factory=list)
    created_at: datetime = field(default_factory=datetime.now)
    resolved_at: Optional[datetime] = None
    
    def to_dict(self) -> dict:
        return {
            "id": self.id,
            "gap_type": self.gap_type.value,
            "description": self.description,
            "severity": self.severity,
            "evidence": self.evidence,
            "suggested_fixes": self.suggested_fixes,
            "created_at": self.created_at.isoformat(),
            "resolved_at": self.resolved_at.isoformat() if self.resolved_at else None
        }


@dataclass
class EvolutionProposal:
    """Propuesta de mejora derivada del análisis."""
    id: str
    title: str
    description: str
    target_gap: Optional[str]           # ID del gap que aborda
    changes: dict                      # Cambios propuestos
    expected_impact: float             # 0.0 - 1.0
    risk_level: float                  # 0.0 - 1.0
    trial_count: int = 0
    success_count: int = 0
    status: str = "proposed"           # proposed, trial, adopted, rejected
    created_at: datetime = field(default_factory=datetime.now)
    
    @property
    def success_rate(self) -> float:
        if self.trial_count == 0:
            return 0.0
        return self.success_count / self.trial_count


@dataclass
class EvolutionMetrics:
    """Métricas de salud del sistema."""
    timestamp: datetime
    total_gaps: int = 0
    open_gaps: int = 0
    critical_gaps: int = 0
    proposals_count: int = 0
    adoption_rate: float = 0.0
    avg_severity: float = 0.0
    coherence_score: float = 1.0       # 0.0 - 1.0
    kg_health_score: float = 1.0      # 0.0 - 1.0
    behavior_score: float = 1.0        # 0.0 - 1.0
    
    def to_dict(self) -> dict:
        return {
            "timestamp": self.timestamp.isoformat(),
            "total_gaps": self.total_gaps,
            "open_gaps": self.open_gaps,
            "critical_gaps": self.critical_gaps,
            "proposals_count": self.proposals_count,
            "adoption_rate": self.adoption_rate,
            "avg_severity": self.avg_severity,
            "coherence_score": self.coherence_score,
            "kg_health_score": self.kg_health_score,
            "behavior_score": self.behavior_score
        }


@dataclass 
class EvolutionConfig:
    """Configuración del sistema de evolución."""
    enabled: bool = True
    interval_seconds: int = 3600       # Cada hora
    min_severity_threshold: float = 0.3
    max_concurrent_trials: int = 3
    trial_duration_seconds: int = 300   # 5 minutos
    max_gaps_stored: int = 100
    auto_adopt_threshold: float = 0.8  # Adoption automática si > 80% éxito
    kg_prune_threshold_days: int = 30
    
    def to_dict(self) -> dict:
        return {
            "enabled": self.enabled,
            "interval_seconds": self.interval_seconds,
            "min_severity_threshold": self.min_severity_threshold,
            "max_concurrent_trials": self.max_concurrent_trials,
            "trial_duration_seconds": self.trial_duration_seconds,
            "max_gaps_stored": self.max_gaps_stored,
            "auto_adopt_threshold": self.auto_adopt_threshold,
            "kg_prune_threshold_days": self.kg_prune_threshold_days
        }


class UnsupervisedEvolution:
    """
    Sistema de evolución no supervisada para Eden.
    
    Opera en ciclos continuos, detectando gaps y proponiendo mejoras
    de forma autónoma. Mantiene un registro de cambios aplicados.
    """
    
    def __init__(
        self,
        kg,  # KnowledgeGraphInterface
        identity_path: str = "/home/ubuntu/eden_identity.json",
        config: Optional[EvolutionConfig] = None
    ):
        self.kg = kg
        self.identity_path = identity_path
        self.config = config or EvolutionConfig()
        
        # Estado interno
        self._gaps: dict[str, EvolutionGap] = {}
        self._proposals: dict[str, EvolutionProposal] = {}
        self._metrics_history: list[EvolutionMetrics] = []
        self._active_trials: dict[str, dict] = {}
        self._running = False
        self._task: Optional[asyncio.Task] = None
        
        # Callbacks para integración con el sistema
        self._on_gap_detected: Optional[Callable[[EvolutionGap], None]] = None
        self._on_proposal_created: Optional[Callable[[EvolutionProposal], None]] = None
        self._on_change_adopted: Optional[Callable[[EvolutionProposal], None]] = None
        
        # Estadísticas
        self._evolution_count = 0
        self._last_evolution: Optional[datetime] = None
        
    @property
    def gaps(self) -> list[EvolutionGap]:
        """Retorna gaps abiertos ordenados por severidad."""
        return sorted(
            [g for g in self._gaps.values() if g.resolved_at is None],
            key=lambda g: g.severity,
            reverse=True
        )
    
    @property
    def proposals(self) -> list[EvolutionProposal]:
        """Retorna propuestas activas."""
        return [p for p in self._proposals.values() if p.status in ("proposed", "trial")]
    
    @property
    def metrics(self) -> EvolutionMetrics:
        """Calcula métricas actuales de salud."""
        open_gaps = [g for g in self._gaps.values() if g.resolved_at is None]
        critical = [g for g in open_gaps if g.severity >= 0.7]
        
        all_proposals = list(self._proposals.values())
        adopted = [p for p in all_proposals if p.status == "adopted"]
        
        return EvolutionMetrics(
            timestamp=datetime.now(),
            total_gaps=len(self._gaps),
            open_gaps=len(open_gaps),
            critical_gaps=len(critical),
            proposals_count=len(all_proposals),
            adoption_rate=len(adopted) / max(1, len(all_proposals)),
            avg_severity=sum(g.severity for g in open_gaps) / max(1, len(open_gaps)),
            coherence_score=self._calculate_coherence_score(),
            kg_health_score=self._calculate_kg_health(),
            behavior_score=self._calculate_behavior_score()
        )
    
    def _load_identity(self) -> dict:
        """Carga identidad desde archivo."""
        try:
            with open(self.identity_path, 'r') as f:
                return json.load(f)
        except Exception:
            return {}
    
    def _save_identity(self, identity: dict) -> None:
        """Guarda identidad a archivo."""
        with open(self.identity_path, 'w') as f:
            json.dump(identity, f, indent=2)
    
    async def start(self) -> None:
        """Inicia el ciclo de evolución."""
        if self._running:
            return
        
        self._running = True
        self._task = asyncio.create_task(self._evolution_loop())
        logger.info("[EVOLUTION] Sistema de evolución iniciado")
    
    async def stop(self) -> None:
        """Detiene el ciclo de evolución."""
        self._running = False
        if self._task:
            self._task.cancel()
            try:
                await self._task
            except asyncio.CancelledError:
                pass
        logger.info("[EVOLUTION] Sistema de evolución detenido")
    
    async def _evolution_loop(self) -> None:
        """Loop principal de evolución."""
        while self._running:
            try:
                await self._run_evolution_cycle()
                self._evolution_count += 1
                self._last_evolution = datetime.now()
            except Exception as e:
                logger.error(f"[EVOLUTION] Error en ciclo: {e}")
            
            await asyncio.sleep(self.config.interval_seconds)
    
    async def _run_evolution_cycle(self) -> None:
        """Ejecuta un ciclo completo de evolución."""
        logger.info(f"[EVOLUTION] Iniciando ciclo #{self._evolution_count + 1}")
        
        # 1. Introspección: analizar estado actual
        phase = EvolutionPhase.INTROSPECTION
        current_state = await self._introspect()
        
        # 2. Diagnosis: identificar gaps
        phase = EvolutionPhase.DIAGNOSIS
        detected_gaps = await self._diagnose(current_state)
        
        for gap in detected_gaps:
            self._gaps[gap.id] = gap
            if self._on_gap_detected:
                self._on_gap_detected(gap)
        
        # Limitar gaps almacenados
        self._prune_old_gaps()
        
        # 3. Proposal: generar propuestas de mejora
        phase = EvolutionPhase.PROPOSAL
        proposals = await self._generate_proposals(detected_gaps, current_state)
        
        for proposal in proposals:
            self._proposals[proposal.id] = proposal
            if self._on_proposal_created:
                self._on_proposal_created(proposal)
        
        # 4. Trial: ejecutar trials para propuestas prometedoras
        phase = EvolutionPhase.TRIAL
        await self._run_trials()
        
        # 5. Adoption: aplicar cambios aprobados
        phase = EvolutionPhase.ADOPTION
        await self._adopt_approved_changes()
        
        # 6. Verification: verificar métricas
        phase = EvolutionPhase.VERIFICATION
        metrics = self.metrics
        self._metrics_history.append(metrics)
        
        logger.info(
            f"[EVOLUTION] Ciclo completado: {len(detected_gaps)} gaps, "
            f"{len(proposals)} propuestas, health={metrics.coherence_score:.2f}"
        )
    
    async def _introspect(self) -> dict:
        """Analiza el estado actual del sistema."""
        identity = self._load_identity()
        
        # Analizar KG
        kg_stats = {
            "nodes_count": 0,
            "edges_count": 0,
            "labels": defaultdict(int),
            "orphan_nodes": [],
            "recent_nodes": []
        }
        
        try:
            from core.knowledge_graph import KGQuery, KGQueryType
            result = await self.kg.query(KGQuery(
                query_type=KGQueryType.SIMPLE,
                cypher="MATCH (n) RETURN n"
            ))
            kg_stats["nodes_count"] = len(result.nodes)
            
            for node in result.nodes:
                kg_stats["labels"][node.label] += 1
                age = (datetime.now() - node.created_at).days
                if age <= 7:
                    kg_stats["recent_nodes"].append(node.id)
            
            kg_stats["edges_count"] = len(result.edges)
        except Exception as e:
            logger.warning(f"[EVOLUTION] Error introspecting KG: {e}")
        
        return {
            "identity": identity,
            "kg_stats": dict(kg_stats),
            "gaps_count": len(self.gaps),
            "proposals_count": len(self.proposals),
            "timestamp": datetime.now().isoformat()
        }
    
    async def _diagnose(self, state: dict) -> list[EvolutionGap]:
        """Identifica gaps en el sistema."""
        gaps = []
        
        # Gap: conocimiento faltante en dominios clave
        kg_stats = state.get("kg_stats", {})
        labels = kg_stats.get("labels", {})
        
        critical_domains = ["task", "decision", "outcome", "reflection"]
        for domain in critical_domains:
            if labels.get(domain, 0) < 5:
                gaps.append(EvolutionGap(
                    id=str(uuid.uuid4()),
                    gap_type=GapType.KNOWLEDGE,
                    description=f"Bajo conocimiento en dominio: {domain}",
                    severity=0.5,
                    evidence=[f"Solo {labels.get(domain, 0)} nodos de tipo {domain}"],
                    suggested_fixes=[
                        f"Ejecutar más tareas en dominio {domain}",
                        f"Reforzar conexiones relacionadas con {domain}"
                    ]
                ))
        
        # Gap: nodos huérfanos
        recent = kg_stats.get("recent_nodes", [])
        if len(recent) == 0 and kg_stats.get("nodes_count", 0) > 0:
            gaps.append(EvolutionGap(
                id=str(uuid.uuid4()),
                gap_type=GapType.STALENESS,
                description="Sin actividad reciente en el KG",
                severity=0.4,
                evidence=["No hay nodos creados en los últimos 7 días"],
                suggested_fixes=[
                    "Ejecutar tareas para generar nuevo conocimiento",
                    "Revisar pipeline de persistencia"
                ]
            ))
        
        # Gap: coherencia de identidad
        identity = state.get("identity", {})
        required_fields = ["name", "core_values", "principles"]
        missing_fields = [f for f in required_fields if f not in identity]
        
        if missing_fields:
            gaps.append(EvolutionGap(
                id=str(uuid.uuid4()),
                gap_type=GapType.COHERENCE,
                description=f"Campos faltantes en identidad: {missing_fields}",
                severity=0.6,
                evidence=[f"Campo '{f}' no encontrado" for f in missing_fields],
                suggested_fixes=["Completar configuración de identidad"]
            ))
        
        # Gap: gaps previos sin resolver
        old_gaps = [g for g in self.gaps if (datetime.now() - g.created_at).days > 7]
        if old_gaps:
            avg_severity = sum(g.severity for g in old_gaps) / len(old_gaps)
            gaps.append(EvolutionGap(
                id=str(uuid.uuid4()),
                gap_type=GapType.BEHAVIOR,
                description=f"{len(old_gaps)} gaps sin resolver por más de 7 días",
                severity=min(0.9, avg_severity + 0.1),
                evidence=[g.description for g in old_gaps[:3]],
                suggested_fixes=[
                    "Revisar manualmente gaps persistentes",
                    "Ajustar umbrales de detección"
                ]
            ))
        
        # Filtrar por severidad mínima
        gaps = [g for g in gaps if g.severity >= self.config.min_severity_threshold]
        
        return gaps
    
    async def _generate_proposals(
        self, 
        gaps: list[EvolutionGap], 
        state: dict
    ) -> list[EvolutionProposal]:
        """Genera propuestas de mejora para los gaps detectados."""
        proposals = []
        
        for gap in gaps:
            # No generar propuestas duplicadas para el mismo gap
            existing = [p for p in self._proposals.values() if p.target_gap == gap.id]
            if existing:
                continue
            
            if gap.gap_type == GapType.KNOWLEDGE:
                proposals.append(EvolutionProposal(
                    id=str(uuid.uuid4()),
                    title=f"Mejorar conocimiento en {gap.description}",
                    description=f"Reforzar el dominio identificado como deficiente",
                    target_gap=gap.id,
                    changes={
                        "action": "reinforce_domain",
                        "domain": gap.description.split(": ")[-1] if ": " in gap.description else "general",
                        "priority": gap.severity
                    },
                    expected_impact=gap.severity,
                    risk_level=0.1
                ))
            
            elif gap.gap_type == GapType.STALENESS:
                proposals.append(EvolutionProposal(
                    id=str(uuid.uuid4()),
                    title="Revitalizar pipeline de conocimiento",
                    description="No hay actividad reciente, verificar sistema",
                    target_gap=gap.id,
                    changes={
                        "action": "check_pipeline",
                        "verify_kg_connection": True,
                        "simulate_activity": True
                    },
                    expected_impact=0.5,
                    risk_level=0.2
                ))
            
            elif gap.gap_type == GapType.COHERENCE:
                proposals.append(EvolutionProposal(
                    id=str(uuid.uuid4()),
                    title="Completar identidad",
                    description="Faltan campos críticos en la identidad",
                    target_gap=gap.id,
                    changes={
                        "action": "initialize_identity_fields",
                        "fields": gap.evidence
                    },
                    expected_impact=gap.severity,
                    risk_level=0.05
                ))
            
            elif gap.gap_type == GapType.BEHAVIOR:
                proposals.append(EvolutionProposal(
                    id=str(uuid.uuid4()),
                    title="Revisar gaps persistentes",
                    description="Gaps de larga duración requieren atención",
                    target_gap=gap.id,
                    changes={
                        "action": "manual_review",
                        "mark_for_human": True
                    },
                    expected_impact=0.7,
                    risk_level=0.0
                ))
        
        return proposals
    
    async def _run_trials(self) -> None:
        """Ejecuta trials para propuestas activas."""
        trial_proposals = [
            p for p in self.proposals 
            if p.status == "proposed" 
            and len([t for t in self._active_trials.values() if t.get("proposal_id")]) < self.config.max_concurrent_trials
        ]
        
        for proposal in trial_proposals[:self.config.max_concurrent_trials]:
            proposal.status = "trial"
            proposal.trial_count += 1
            
            trial_id = str(uuid.uuid4())
            self._active_trials[trial_id] = {
                "proposal_id": proposal.id,
                "started_at": datetime.now(),
                "changes": proposal.changes
            }
            
            # Ejecutar trial en background
            asyncio.create_task(self._execute_trial(trial_id, proposal))
    
    async def _execute_trial(self, trial_id: str, proposal: EvolutionProposal) -> None:
        """Ejecuta un trial específico."""
        try:
            await asyncio.sleep(min(10, self.config.trial_duration_seconds))
            
            # Simular evaluación del trial
            # En producción, esto evaluaría métricas reales
            success = random.random() < proposal.expected_impact
            
            if success:
                proposal.success_count += 1
                proposal.status = "adopted" if proposal.success_rate >= self.config.auto_adopt_threshold else "proposed"
                
                if self._on_change_adopted:
                    self._on_change_adopted(proposal)
            else:
                proposal.status = "proposed"
            
        except Exception as e:
            logger.error(f"[EVOLUTION] Trial {trial_id} falló: {e}")
            proposal.status = "proposed"
        finally:
            if trial_id in self._active_trials:
                del self._active_trials[trial_id]
    
    async def _adopt_approved_changes(self) -> None:
        """Aplica cambios de propuestas adoptadas."""
        for proposal in [p for p in self._proposals.values() if p.status == "adopted"]:
            if proposal.changes.get("action") == "initialize_identity_fields":
                identity = self._load_identity()
                
                # Inicializar campos faltantes
                if "core_values" not in identity:
                    identity["core_values"] = ["autonomía", "mejora continua", "coherencia"]
                if "principles" not in identity:
                    identity["principles"] = [
                        "Priorizar calidad sobre cantidad",
                        "Mantener transparencia en decisiones",
                        "Evolucionar basado en evidencia"
                    ]
                if "behavior_patterns" not in identity:
                    identity["behavior_patterns"] = {}
                
                self._save_identity(identity)
                logger.info(f"[EVOLUTION] Identidad actualizada con campos nuevos")
            
            elif proposal.changes.get("action") == "check_pipeline":
                # Verificar que el KG responde
                try:
                    from core.knowledge_graph import KGQuery, KGQueryType
                    await self.kg.query(KGQuery(query_type=KGQueryType.SIMPLE))
                    logger.info("[EVOLUTION] KG respondiendo correctamente")
                except Exception as e:
                    logger.warning(f"[EVOLUTION] KG tiene problemas: {e}")
    
    def _calculate_coherence_score(self) -> float:
        """Calcula score de coherencia de identidad."""
        identity = self._load_identity()
        
        required = ["name", "core_values", "principles"]
        present = sum(1 for f in required if f in identity and identity[f])
        
        if present == 0:
            return 0.3
        
        # Penalizar gaps abiertos relacionados con coherencia
        coherence_gaps = [g for g in self.gaps if g.gap_type == GapType.COHERENCE]
        penalty = sum(g.severity for g in coherence_gaps) * 0.2
        
        return min(1.0, (present / len(required)) - penalty + 0.5)
    
    def _calculate_kg_health(self) -> float:
        """Calcula score de salud del KG."""
        open_gaps = self.gaps
        staleness_gaps = [g for g in open_gaps if g.gap_type == GapType.STALENESS]
        connection_gaps = [g for g in open_gaps if g.gap_type == GapType.CONNECTION]
        
        penalty = len(staleness_gaps) * 0.1 + len(connection_gaps) * 0.15
        
        return max(0.0, 1.0 - penalty)
    
    def _calculate_behavior_score(self) -> float:
        """Calcula score de comportamiento."""
        adopted = [p for p in self._proposals.values() if p.status == "adopted"]
        if not adopted:
            return 0.5
        
        avg_success = sum(p.success_rate for p in adopted) / len(adopted)
        return min(1.0, avg_success + 0.3)
    
    def _prune_old_gaps(self) -> None:
        """Elimina gaps muy antiguos si excede el límite."""
        if len(self._gaps) <= self.config.max_gaps_stored:
            return
        
        # Ordenar por severidad y mantener los más severos
        sorted_gaps = sorted(
            self._gaps.items(),
            key=lambda x: x[1].severity,
            reverse=True
        )
        
        self._gaps = dict(sorted_gaps[:self.config.max_gaps_stored])
    
    def resolve_gap(self, gap_id: str) -> bool:
        """Marca un gap como resuelto."""
        if gap_id in self._gaps:
            self._gaps[gap_id].resolved_at = datetime.now()
            return True
        return False
    
    def get_evolution_report(self) -> dict:
        """Genera reporte del estado de evolución."""
        return {
            "evolution_count": self._evolution_count,
            "last_evolution": self._last_evolution.isoformat() if self._last_evolution else None,
            "current_metrics": self.metrics.to_dict(),
            "open_gaps": [g.to_dict() for g in self.gaps],
            "active_proposals": [p.__dict__ for p in self.proposals],
            "config": self.config.to_dict()
        }
    
    async def trigger_evolution(self) -> dict:
        """Fuerza un ciclo de evolución inmediato."""
        await self._run_evolution_cycle()
        return self.get_evolution_report()


# Función helper para crear instancia con configuración por defecto
def create_evolution_system(kg, **kwargs) -> UnsupervisedEvolution:
    """Factory para crear sistema de evolución."""
    config = EvolutionConfig(**kwargs) if kwargs else None
    return UnsupervisedEvolution(kg, config=config)
