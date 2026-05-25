"""
Autoevaluación de Identidad - Sistema de verificación y refinamiento de identidad para Eden.

Este módulo implementa mecanismos de autoevaluación que permiten a Eden:
- Verificar coherencia entre identidad declarada y comportamiento real
- Detectar drifts de personalidad o valores
- Proponer ajustes a la identidad basados en experiencia
- Mantener un log de evolución de identidad
- Validar decisiones contra principios establecidos
"""

from __future__ import annotations

import json
import logging
import uuid
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from typing import Optional, Callable, Any
from collections import defaultdict

logger = logging.getLogger(__name__)


class IdentityDimension(Enum):
    """Dimensiones de la identidad a evaluar."""
    NAME = "name"
    VALUES = "values"
    PRINCIPLES = "principles"
    BEHAVIORS = "behaviors"
    CAPABILITIES = "capabilities"
    GOALS = "goals"
    LIMITATIONS = "limitations"


class DriftType(Enum):
    """Tipos de drift detectados."""
    VALUE_DRIFT = "value_drift"         # Valores se alejan de los declarados
    BEHAVIOR_DRIFT = "behavior_drift"   # Comportamiento inconsistente
    CAPABILITY_DRIFT = "capability_drift"  # Capacidades declaradas vs reales
    COHERENCE_BREAK = "coherence_break"  # Inconsistencia interna


@dataclass
class IdentityClaim:
    """Una declaración sobre la identidad (valor, principio, etc)."""
    id: str
    dimension: IdentityDimension
    claim: str
    source: str = "user"                # user, self_declared, inferred
    confidence: float = 1.0             # 0.0 - 1.0
    created_at: datetime = field(default_factory=datetime.now)
    last_verified: Optional[datetime] = None
    
    def to_dict(self) -> dict:
        return {
            "id": self.id,
            "dimension": self.dimension.value,
            "claim": self.claim,
            "source": self.source,
            "confidence": self.confidence,
            "created_at": self.created_at.isoformat(),
            "last_verified": self.last_verified.isoformat() if self.last_verified else None
        }


@dataclass
class IdentityViolation:
    """Una violación detectada de la identidad."""
    id: str
    dimension: IdentityDimension
    claim_id: Optional[str]            # Claim que fue violado
    description: str
    severity: float                    # 0.0 - 1.0
    evidence: list[str] = field(default_factory=list)
    detected_at: datetime = field(default_factory=datetime.now)
    resolved_at: Optional[datetime] = None
    
    def to_dict(self) -> dict:
        return {
            "id": self.id,
            "dimension": self.dimension.value,
            "claim_id": self.claim_id,
            "description": self.description,
            "severity": self.severity,
            "evidence": self.evidence,
            "detected_at": self.detected_at.isoformat(),
            "resolved_at": self.resolved_at.isoformat() if self.resolved_at else None
        }


@dataclass
class BehaviorObservation:
    """Una observación de comportamiento."""
    id: str
    description: str
    timestamp: datetime = field(default_factory=datetime.now)
    context: dict = field(default_factory=dict)
    aligned_with_identity: Optional[bool] = None  # None = no evaluado
    
    def to_dict(self) -> dict:
        return {
            "id": self.id,
            "description": self.description,
            "timestamp": self.timestamp.isoformat(),
            "context": self.context,
            "aligned_with_identity": self.aligned_with_identity
        }


@dataclass
class IdentityReport:
    """Reporte de autoevaluación."""
    timestamp: datetime
    coherence_score: float             # 0.0 - 1.0
    drift_score: float                # 0.0 - 1.0 (inverso de estabilidad)
    verified_claims: int
    total_claims: int
    open_violations: int
    new_claims_added: int
    recommendations: list[str]
    
    def to_dict(self) -> dict:
        return {
            "timestamp": self.timestamp.isoformat(),
            "coherence_score": self.coherence_score,
            "drift_score": self.drift_score,
            "verified_claims": self.verified_claims,
            "total_claims": self.total_claims,
            "open_violations": self.open_violations,
            "new_claims_added": self.new_claims_added,
            "recommendations": self.recommendations
        }


@dataclass
class SelfEvaluationConfig:
    """Configuración del sistema de autoevaluación."""
    enabled: bool = True
    evaluation_interval_seconds: int = 1800  # Cada 30 minutos
    coherence_threshold: float = 0.7          # Umbral mínimo de coherencia
    drift_threshold: float = 0.3              # Umbral de drift aceptable
    max_observations: int = 1000
    max_violations_stored: int = 50
    auto_resolve_threshold: float = 0.2     # Auto-resolver si severity < 0.2
    
    def to_dict(self) -> dict:
        return {
            "enabled": self.enabled,
            "evaluation_interval_seconds": self.evaluation_interval_seconds,
            "coherence_threshold": self.coherence_threshold,
            "drift_threshold": self.drift_threshold,
            "max_observations": self.max_observations,
            "max_violations_stored": self.max_violations_stored,
            "auto_resolve_threshold": self.auto_resolve_threshold
        }


class IdentitySelfEvaluator:
    """
    Sistema de autoevaluación de identidad para Eden.
    
    Monitorea coherencia entre identidad declarada y comportamiento,
    detecta drifts, y propone ajustes cuando es necesario.
    """
    
    def __init__(
        self,
        identity_path: str = "/home/ubuntu/eden_identity.json",
        config: Optional[SelfEvaluationConfig] = None
    ):
        self.identity_path = identity_path
        self.config = config or SelfEvaluationConfig()
        
        # Estado interno
        self._claims: dict[str, IdentityClaim] = {}
        self._violations: dict[str, IdentityViolation] = {}
        self._observations: list[BehaviorObservation] = []
        self._identity_log: list[dict] = []
        
        # Callbacks
        self._on_violation_detected: Optional[Callable[[IdentityViolation], None]] = None
        self._on_identity_change: Optional[Callable[[dict, dict], None]] = None  # old, new
        
        # Estado del loop
        self._running = False
        self._task = None
        self._evaluation_count = 0
        
        # Inicializar desde archivo de identidad
        self._initialize_from_identity()
    
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
    
    def _initialize_from_identity(self) -> None:
        """Inicializa claims desde archivo de identidad existente."""
        identity = self._load_identity()
        
        # Claims de valores core
        for i, value in enumerate(identity.get("core_values", [])):
            claim_id = str(uuid.uuid4())
            self._claims[claim_id] = IdentityClaim(
                id=claim_id,
                dimension=IdentityDimension.VALUES,
                claim=value,
                source="user",
                confidence=1.0
            )
        
        # Claims de principios
        for i, principle in enumerate(identity.get("principles", [])):
            claim_id = str(uuid.uuid4())
            self._claims[claim_id] = IdentityClaim(
                id=claim_id,
                dimension=IdentityDimension.PRINCIPLES,
                claim=principle,
                source="user",
                confidence=1.0
            )
        
        # Claims de nombre
        if "name" in identity:
            claim_id = str(uuid.uuid4())
            self._claims[claim_id] = IdentityClaim(
                id=claim_id,
                dimension=IdentityDimension.NAME,
                claim=identity["name"],
                source="user",
                confidence=1.0
            )
        
        logger.info(f"[SELF-EVAL] Inicializado con {len(self._claims)} claims")
    
    @property
    def claims(self) -> list[IdentityClaim]:
        """Retorna todos los claims."""
        return list(self._claims.values())
    
    @property
    def violations(self) -> list[IdentityViolation]:
        """Retorna violaciones abiertas."""
        return [v for v in self._violations.values() if v.resolved_at is None]
    
    @property
    def observations(self) -> list[BehaviorObservation]:
        """Retorna observaciones recientes."""
        return self._observations[-100:]
    
    async def start(self) -> None:
        """Inicia el loop de autoevaluación."""
        if self._running:
            return
        
        self._running = True
        self._task = asyncio.create_task(self._evaluation_loop())
        logger.info("[SELF-EVAL] Autoevaluación iniciada")
    
    async def stop(self) -> None:
        """Detiene el loop de autoevaluación."""
        self._running = False
        if self._task:
            self._task.cancel()
            try:
                await self._task
            except asyncio.CancelledError:
                pass
        logger.info("[SELF-EVAL] Autoevaluación detenida")
    
    async def _evaluation_loop(self) -> None:
        """Loop principal de evaluación."""
        import asyncio
        
        while self._running:
            try:
                await self._run_evaluation()
                self._evaluation_count += 1
            except Exception as e:
                logger.error(f"[SELF-EVAL] Error en evaluación: {e}")
            
            await asyncio.sleep(self.config.evaluation_interval_seconds)
    
    async def _run_evaluation(self) -> IdentityReport:
        """Ejecuta una evaluación completa."""
        logger.info(f"[SELF-EVAL] Ejecutando evaluación #{self._evaluation_count + 1}")
        
        # 1. Verificar coherencia de claims
        coherence_score = self._verify_coherence()
        
        # 2. Detectar drifts
        drift_score = await self._detect_drifts()
        
        # 3. Evaluar observaciones recientes
        new_violations = await self._evaluate_observations()
        
        # 4. Actualizar timestamps de verificación
        self._update_verification_timestamps()
        
        # 5. Auto-resolver violaciones menores
        auto_resolved = self._auto_resolve_violations()
        
        # 6. Generar recomendaciones
        recommendations = self._generate_recommendations(coherence_score, drift_score)
        
        report = IdentityReport(
            timestamp=datetime.now(),
            coherence_score=coherence_score,
            drift_score=drift_score,
            verified_claims=sum(1 for c in self._claims.values() if c.last_verified is not None),
            total_claims=len(self._claims),
            open_violations=len(self.violations),
            new_claims_added=new_violations,
            recommendations=recommendations
        )
        
        logger.info(
            f"[SELF-EVAL] Evaluación completada: coherence={coherence_score:.2f}, "
            f"drift={drift_score:.2f}, violations={len(self.violations)}"
        )
        
        return report
    
    def _verify_coherence(self) -> float:
        """Verifica coherencia entre claims."""
        if not self._claims:
            return 1.0
        
        # Agrupar por dimensión
        by_dimension: dict[IdentityDimension, list[IdentityClaim]] = defaultdict(list)
        for claim in self._claims.values():
            by_dimension[claim.dimension].append(claim)
        
        coherence_scores = []
        
        for dimension, claims in by_dimension.items():
            if len(claims) <= 1:
                coherence_scores.append(1.0)
                continue
            
            # Claims del mismo tipo deberían ser consistentes
            # Por ahora, asumimos que todos son coherentes
            # En producción, usar embeddings para comparar semántica
            dimension_score = 0.9  # Placeholder
            
            # Penalizar por violaciones en esta dimensión
            dimension_violations = [
                v for v in self.violations 
                if v.dimension == dimension
            ]
            violation_penalty = sum(v.severity for v in dimension_violations) / len(claims)
            
            coherence_scores.append(max(0.0, dimension_score - violation_penalty))
        
        return sum(coherence_scores) / len(coherence_scores)
    
    async def _detect_drifts(self) -> float:
        """Detecta drifts de identidad."""
        identity = self._load_identity()
        
        total_drift = 0.0
        drift_count = 0
        
        # Verificar drift de valores
        declared_values = set(identity.get("core_values", []))
        observed_values = set()
        
        for obs in self._observations:
            # Extraer valores observados del contexto
            if "values_exhibited" in obs.context:
                observed_values.update(obs.context["values_exhibited"])
        
        if declared_values:
            # Calcular drift: qué valores declarados no se observan
            missing_values = declared_values - observed_values
            value_drift = len(missing_values) / len(declared_values)
            
            if value_drift > self.config.drift_threshold:
                violation_id = str(uuid.uuid4())
                self._violations[violation_id] = IdentityViolation(
                    id=violation_id,
                    dimension=IdentityDimension.VALUES,
                    description=f"Valores declarados no observados: {missing_values}",
                    severity=value_drift,
                    evidence=[f"'{v}' declarado pero no observado" for v in missing_values]
                )
            
            total_drift += value_drift
            drift_count += 1
        
        # Verificar drift de comportamiento
        behavior_violations = [v for v in self.violations if v.dimension == IdentityDimension.BEHAVIORS]
        if behavior_violations:
            behavior_drift = sum(v.severity for v in behavior_violations) / len(behavior_violations)
            total_drift += behavior_drift
            drift_count += 1
        
        return total_drift / max(1, drift_count)
    
    async def _evaluate_observations(self) -> int:
        """Evalúa observaciones recientes contra identidad."""
        identity = self._load_identity()
        declared_values = set(identity.get("core_values", []))
        
        new_violations_count = 0
        
        for obs in self._observations:
            if obs.aligned_with_identity is not None:
                continue  # Ya evaluada
            
            # Evaluación simple: verificar si hay evidencia de conflicto
            conflicts = []
            
            for value in declared_values:
                if f"not_{value.lower().replace(' ', '_')}" in obs.description.lower():
                    conflicts.append(value)
            
            if conflicts:
                violation_id = str(uuid.uuid4())
                severity = min(1.0, len(conflicts) * 0.3)
                
                self._violations[violation_id] = IdentityViolation(
                    id=violation_id,
                    dimension=IdentityDimension.BEHAVIORS,
                    description=f"Posible conflicto con valores: {conflicts}",
                    severity=severity,
                    evidence=[obs.description]
                )
                new_violations_count += 1
                
                if self._on_violation_detected:
                    self._on_violation_detected(self._violations[violation_id])
            
            obs.aligned_with_identity = len(conflicts) == 0
        
        return new_violations_count
    
    def _update_verification_timestamps(self) -> None:
        """Actualiza timestamps de verificación de claims."""
        for claim in self._claims.values():
            if claim.last_verified is None:
                claim.last_verified = datetime.now()
    
    def _auto_resolve_violations(self) -> int:
        """Auto-resuelve violaciones de baja severidad."""
        resolved = 0
        to_remove = []
        
        for violation in self.violations:
            if violation.severity < self.config.auto_resolve_threshold:
                violation.resolved_at = datetime.now()
                resolved += 1
        
        return resolved
    
    def _generate_recommendations(self, coherence: float, drift: float) -> list[str]:
        """Genera recomendaciones basadas en el estado actual."""
        recommendations = []
        
        if coherence < self.config.coherence_threshold:
            recommendations.append(
                f"Coherencia baja ({coherence:.2f}). Revisar conflictos en identidad."
            )
        
        if drift > self.config.drift_threshold:
            recommendations.append(
                f"Drift detectado ({drift:.2f}). Ajustar comportamiento para alinearse con valores."
            )
        
        if len(self.violations) > 5:
            recommendations.append(
                f"Múltiples violaciones activas ({len(self.violations)}). "
                "Considerar revisión de principios."
            )
        
        if len(self._claims) < 5:
            recommendations.append(
                "Pocos claims definidos. Considerar expandir identidad."
            )
        
        if not recommendations:
            recommendations.append("Identidad coherente. Continuar monitoreando.")
        
        return recommendations
    
    def add_observation(
        self, 
        description: str, 
        context: Optional[dict] = None
    ) -> BehaviorObservation:
        """Registra una observación de comportamiento."""
        obs = BehaviorObservation(
            id=str(uuid.uuid4()),
            description=description,
            context=context or {}
        )
        
        self._observations.append(obs)
        
        # Limitar observaciones almacenadas
        if len(self._observations) > self.config.max_observations:
            self._observations = self._observations[-self.config.max_observations:]
        
        return obs
    
    def add_claim(
        self,
        dimension: IdentityDimension,
        claim: str,
        source: str = "self_declared",
        confidence: float = 0.7
    ) -> IdentityClaim:
        """Agrega un nuevo claim a la identidad."""
        claim_id = str(uuid.uuid4())
        
        new_claim = IdentityClaim(
            id=claim_id,
            dimension=dimension,
            claim=claim,
            source=source,
            confidence=confidence
        )
        
        self._claims[claim_id] = new_claim
        
        # Log del cambio
        self._identity_log.append({
            "timestamp": datetime.now().isoformat(),
            "action": "claim_added",
            "claim": new_claim.to_dict()
        })
        
        return new_claim
    
    def update_identity_field(
        self, 
        field: str, 
        value: Any,
        trigger_evaluation: bool = True
    ) -> None:
        """Actualiza un campo de la identidad."""
        old_identity = self._load_identity()
        new_identity = old_identity.copy()
        
        # Actualizar el campo
        if field in new_identity:
            old_value = new_identity[field]
            if isinstance(old_value, list) and isinstance(value, list):
                # Merge lists
                new_identity[field] = list(set(old_value + value))
            else:
                new_identity[field] = value
        else:
            new_identity[field] = value
        
        self._save_identity(new_identity)
        
        # Log del cambio
        self._identity_log.append({
            "timestamp": datetime.now().isoformat(),
            "action": "identity_updated",
            "field": field,
            "old_value": old_identity.get(field),
            "new_value": value
        })
        
        # Notificar si hay callback
        if self._on_identity_change and trigger_evaluation:
            self._on_identity_change(old_identity, new_identity)
        
        logger.info(f"[SELF-EVAL] Campo '{field}' actualizado")
    
    def resolve_violation(self, violation_id: str, resolution: str = "manual") -> bool:
        """Marca una violación como resuelta."""
        if violation_id in self._violations:
            self._violations[violation_id].resolved_at = datetime.now()
            
            self._identity_log.append({
                "timestamp": datetime.now().isoformat(),
                "action": "violation_resolved",
                "violation_id": violation_id,
                "resolution": resolution
            })
            
            return True
        return False
    
    def propose_identity_adjustment(
        self, 
        claim_id: Optional[str] = None,
        dimension: Optional[IdentityDimension] = None,
        reason: str = ""
    ) -> dict:
        """Proponer un ajuste a la identidad."""
        proposal = {
            "id": str(uuid.uuid4()),
            "timestamp": datetime.now().isoformat(),
            "claim_id": claim_id,
            "dimension": dimension.value if dimension else None,
            "reason": reason,
            "status": "pending"
        }
        
        self._identity_log.append({
            "timestamp": datetime.now().isoformat(),
            "action": "adjustment_proposed",
            "proposal": proposal
        })
        
        return proposal
    
    def get_identity_status(self) -> dict:
        """Obtiene estado actual de la identidad."""
        return {
            "total_claims": len(self._claims),
            "claims_by_dimension": {
                dim.value: len([c for c in self._claims.values() if c.dimension == dim])
                for dim in IdentityDimension
            },
            "open_violations": len(self.violations),
            "violations_by_dimension": {
                dim.value: len([v for v in self.violations if v.dimension == dim])
                for dim in IdentityDimension
            },
            "recent_observations": len(self._observations[-100:]),
            "identity_log_entries": len(self._identity_log)
        }
    
    async def force_evaluation(self) -> IdentityReport:
        """Fuerza una evaluación inmediata."""
        return await self._run_evaluation()
    
    def get_identity_report(self) -> dict:
        """Genera reporte completo de identidad."""
        return {
            "status": self.get_identity_status(),
            "coherence_score": self._verify_coherence(),
            "claims": [c.to_dict() for c in self._claims.values()],
            "violations": [v.to_dict() for v in self.violations],
            "recent_observations": [o.to_dict() for o in self.observations],
            "identity_log": self._identity_log[-50:],  # Últimos 50 entries
            "config": self.config.to_dict()
        }


# Factory function
def create_self_evaluator(identity_path: str = "/home/ubuntu/eden_identity.json", **kwargs) -> IdentitySelfEvaluator:
    """Factory para crear evaluador con configuración."""
    config = SelfEvaluationConfig(**kwargs) if kwargs else None
    return IdentitySelfEvaluator(identity_path, config)
