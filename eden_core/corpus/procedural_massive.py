import random
import sys

random.seed(42)

# Templates por dominio con slots
TEMPLATES = {
    "physical": [
        "{obj} cae al suelo porque la gravedad lo atrae",
        "{obj} se calienta porque {source} emite calor",
        "{obj} se enfria porque pierde energia termica",
        "{obj} flota en {medium} porque es menos denso",
        "{obj} se rompe porque la fuerza supera su resistencia",
        "{obj} acelera porque una fuerza neta actua sobre el",
        "{obj} se evapora porque el calor aumenta la energia cinetica",
        "{obj} congela porque la temperatura desciende bajo cero",
        "{obj} emite luz porque sus atomos excitados liberan fotones",
        "{obj} conduce electricidad porque tiene electrones libres",
    ],
    "biological": [
        "{organism} respira porque sus celulas necesitan oxigeno",
        "{organism} come porque requiere energia para vivir",
        "{organism} crece porque sus celulas se dividen",
        "{organism} se reproduce porque perpetua su especie",
        "{organism} duerme porque su cerebro necesita descansar",
        "{organism} siente dolor porque sus nervios detectan danos",
        "{organism} envejece porque sus telomeros se acortan",
        "{organism} emigra porque busca recursos o refugio",
        "{organism} muta porque su ADN sufre alteraciones",
        "{organism} adapta porque la seleccion natural favorece cambios",
    ],
    "cognitive": [
        "{agent} piensa porque procesa informacion simbolica",
        "{agent} recuerda porque sus neuronas fortalecen sinapsis",
        "{agent} aprende porque modifica sus conexiones internas",
        "{agent} olvida porque la memoria no se consolida",
        "{agent} decide porque evalua opciones y consecuencias",
        "{agent} planifica porque anticipa estados futuros",
        "{agent} atiende porque filtra estimulos irrelevantes",
        "{agent} razona porque combina premisas logicas",
        "{agent} imagina porque simula escenarios mentales",
        "{agent} comprende porque integra patrones previos",
    ],
    "social": [
        "{agent} coopera porque beneficia mutuamente",
        "{agent} comunica porque transmite informacion",
        "{agent} compite porque los recursos son limitados",
        "{agent} comparte porque aumenta la confianza",
        "{agent} miente porque oculta la verdad",
        "{agent} confia porque previo comportamiento fue honesto",
        "{agent} lidera porque otros siguen su ejemplo",
        "{agent} aprende de otros porque imita comportamiento exitoso",
        "{agent} castiga porque disuade acciones daninas",
        "{agent} celebra porque refuerza vinculos sociales",
    ],
    "causal_complex": [
        "si {cause} entonces {effect} porque {mechanism}",
        "{cause} y {cause2} juntos producen {effect}",
        "aunque {cause} intenta {effect} falla porque {blocker}",
        "{effect} ocurre porque primero {cause} y luego {cause2}",
        "cuando {cause} aumenta entonces {effect} disminuye",
        "{cause} causa {effect} solo si {condition}",
        "{effect} sucede siempre que {cause} y {cause2} coinciden",
        "sin {cause} no hay {effect} porque dependencia directa",
    ],
}

SLOTS = {
    "obj": ["la piedra", "el agua", "el metal", "el vidrio", "el hielo", "el vapor", "la roca", "el fuego", "la luz", "el sonido", "la electricidad", "el iman", "el aire", "el oxigeno", "el carbon"],
    "source": ["el sol", "el fuego", "la reaccion quimica", "el motor", "el cuerpo humano"],
    "medium": ["el agua", "el aire", "el aceite", "el alcohol"],
    "organism": ["la bacteria", "la planta", "el animal", "el humano", "la celula", "el virus", "el hongo", "el pez", "el pajaro", "el mamifero"],
    "agent": ["el cerebro", "la mente", "el sistema", "la persona", "el robot", "la IA", "el nino", "el adulto", "el experto", "el estudiante"],
    "cause": ["la temperatura", "la presion", "la luz", "la gravedad", "la energia", "la concentracion", "la velocidad", "el tiempo"],
    "cause2": ["la humedad", "la friccion", "la tension", "la carga", "el campo magnetico", "la concentracion"],
    "effect": ["la expansion", "la contraccion", "la transformacion", "la destruccion", "la creacion", "el movimiento", "el cambio"],
    "mechanism": ["la energia se transfiere", "las particulas colisionan", "el equilibrio se rompe", "la estructura cede"],
    "blocker": ["la resistencia es alta", "la energia es insuficiente", "el medio se opone", "la entropia aumenta"],
    "condition": ["la temperatura es alta", "la presion es adecuada", "hay suficiente energia", "el sistema esta estable"],
}

def fill(template):
    s = template
    for key, opts in SLOTS.items():
        while "{" + key + "}" in s:
            s = s.replace("{" + key + "}", random.choice(opts), 1)
    return s

def generate(n):
    sentences = []
    all_templates = []
    for domain, temps in TEMPLATES.items():
        for t in temps:
            all_templates.append((domain, t))
    for i in range(n):
        domain, t = random.choice(all_templates)
        s = fill(t)
        sentences.append(s)
    return sentences

if __name__ == "__main__":
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 5000
    sentences = generate(n)
    for s in sentences:
        print(s)
