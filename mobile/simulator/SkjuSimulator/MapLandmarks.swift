import MapKit


func generateStationName(at coordinate: CLLocationCoordinate2D) async -> String {
    guard let items = try? await nearestLandmarks(to: coordinate),
          let nearest = items.first,
          let landmarkName = nearest.name,
          let prefix = StationAdjectives.all.randomElement(),
          let suffix = StationNouns.all.randomElement()
    else { return randomStationName() }

    return "\(prefix) \(landmarkName) \(suffix)"
}

private func nearestLandmarks(to coordinate: CLLocationCoordinate2D) async throws -> [MKMapItem] {
    let radius: CLLocationDistance = 1000
    let request = MKLocalSearch.Request()
    request.region = MKCoordinateRegion(
        center: coordinate,
        latitudinalMeters:  radius,
        longitudinalMeters: radius
    )
    request.resultTypes = .pointOfInterest
    request.pointOfInterestFilter = MKPointOfInterestFilter(including: [
        .nationalPark,
        .park,
        .landmark,
        .beach,
    ])

    let search  = MKLocalSearch(request: request)
    let results = try await search.start()

    return results.mapItems
}

private func randomStationName() -> String {
    return UUID().uuidString
}

private enum StationNouns {
    static let all: [String] = ["Array", "Station", "Sensor", "Monitor", "Node", "Post"]
}

private enum StationAdjectives {
    static let all: [String] = [
        // Directional / positional
        "Northern", "Southern", "Eastern", "Western",
        "Northeastern", "Northwestern", "Southeastern", "Southwestern",
        "Upper", "Lower", "Inner", "Outer",
        "Central", "Peripheral", "Proximal", "Distal",
        "Near", "Far", "Deep", "Shallow",
        "High", "Low", "Mid", "Trans",

        // Elevation / terrain
        "Alpine", "Subalpine", "Montane", "Highland",
        "Upland", "Lowland", "Elevated", "Summit",
        "Ridgeline", "Clifftop", "Plateau", "Escarpment",
        "Foothill", "Piedmont", "Tableland", "Terrace",
        "Bluff", "Crestline", "Ravine", "Canyon",
        "Valley", "Gorge", "Basin", "Depression",
        "Alluvial", "Deltaic", "Floodplain", "Riparian",

        // Coastal / marine
        "Coastal", "Littoral", "Shoreline", "Tidal",
        "Offshore", "Inshore", "Pelagic", "Abyssal",
        "Oceanic", "Maritime", "Estuarine", "Lagoonal",
        "Cliffside", "Beachfront", "Dune", "Barrier",
        "Insular", "Archipelagic", "Atoll", "Reef",

        // Geological / structural
        "Ridge", "Fault", "Rift", "Graben",
        "Horst", "Anticline", "Syncline", "Monocline",
        "Thrust", "Shear", "Fold", "Uplift",
        "Subsidence", "Volcanic", "Magmatic", "Plutonic",
        "Intrusive", "Extrusive", "Tectonic", "Seismic",
        "Lithic", "Granitic", "Basaltic", "Rhyolitic",
        "Andesitic", "Obsidian", "Pumice", "Pyroclastic",
        "Sedimentary", "Metamorphic", "Igneous", "Crystalline",
        "Stratified", "Foliated", "Fractured", "Jointed",
        "Porous", "Permeable", "Consolidated", "Unconsolidated",

        // Rock / mineral types
        "Granite", "Marble", "Slate", "Schist",
        "Gneiss", "Quartzite", "Limestone", "Sandstone",
        "Shale", "Dolomite", "Chert", "Flint",
        "Obsidian", "Feldspar", "Mica", "Quartz",
        "Calcite", "Pyrite", "Magnetite", "Hematite",

        // Hydrological
        "Glacial", "Periglacial", "Subglacial", "Proglacial",
        "Fluvial", "Lacustrine", "Karst", "Speleological",
        "Artesian", "Phreatic", "Vadose", "Saline",
        "Freshwater", "Brackish", "Thermal", "Hydrothermal",
        "Geyser", "Fumarolic", "Solfataric", "Mofette",

        // Atmospheric / meteorological
        "Windward", "Leeward", "Temperate", "Boreal",
        "Arctic", "Subarctic", "Tropical", "Subtropical",
        "Arid", "Semiarid", "Humid", "Subhumid",
        "Monsoon", "Cyclonic", "Anticyclonic", "Frontal",
        "Squall", "Gale", "Mistral", "Tramontane",
        "Foehn", "Chinook", "Sirocco", "Harmattan",
        "Polar", "Equatorial", "Continental", "Oceanic",

        // Biome / vegetation
        "Tundra", "Taiga", "Boreal", "Temperate",
        "Rainforest", "Savanna", "Steppe", "Prairie",
        "Chaparral", "Maquis", "Scrubland", "Moorland",
        "Heathland", "Marshland", "Wetland", "Peatland",
        "Mangrove", "Kelp", "Coral", "Deciduous",
        "Coniferous", "Mixed", "Old-growth", "Riparian",
        "Subalpine", "Montane", "Lowland", "Cloud",

        // Colour / appearance
        "Black", "White", "Grey", "Silver",
        "Golden", "Amber", "Crimson", "Scarlet",
        "Azure", "Cobalt", "Indigo", "Slate",
        "Ochre", "Sienna", "Umber", "Ivory",
        "Obsidian", "Onyx", "Pearl", "Iron",
        "Copper", "Bronze", "Russet", "Tawny",
        "Emerald", "Jade", "Sage", "Olive",
        "Ashen", "Dusky", "Pallid", "Stark",

        // Size / scale
        "Grand", "Great", "Major", "Minor",
        "Micro", "Macro", "Vast", "Narrow",
        "Broad", "Wide", "Slim", "Thin",
        "Massive", "Colossal", "Towering", "Looming",
        "Immense", "Gigantic", "Tiny", "Compact",

        // Temporal / age
        "Ancient", "Archaic", "Primordial", "Precambrian",
        "Paleozoic", "Mesozoic", "Cenozoic", "Quaternary",
        "Pleistocene", "Holocene", "Neogene", "Paleogene",
        "Cretaceous", "Jurassic", "Triassic", "Permian",
        "Carboniferous", "Devonian", "Silurian", "Ordovician",
        "Cambrian", "Proterozoic", "Archaean", "Hadean",
        "Fossil", "Relict", "Remnant", "Vestigial",
        "Recent", "Modern", "Active", "Dormant",
        "Extinct", "Buried", "Exhumed", "Eroded",

        // Physical properties
        "Dense", "Porous", "Rigid", "Flexible",
        "Brittle", "Ductile", "Elastic", "Plastic",
        "Stable", "Unstable", "Dynamic", "Static",
        "Hot", "Cold", "Warm", "Cool",
        "Frozen", "Thawed", "Melting", "Solidified",
        "Pressurised", "Depressed", "Stressed", "Relaxed",
        "Saturated", "Dry", "Moist", "Wet",

        // Luminous / solar
        "Solar", "Lunar", "Stellar", "Astral",
        "Radiant", "Luminous", "Shadowed", "Eclipsed",
        "Sunlit", "Moonlit", "Twilight", "Midnight",
        "Dawn", "Dusk", "Zenith", "Nadir",

        // Wind / storm
        "Stormy", "Calm", "Breezy", "Gusty",
        "Turbulent", "Serene", "Fierce", "Gentle",
        "Howling", "Whispering", "Roaring", "Still",
        "Tempestuous", "Placid", "Violent", "Tranquil",

        // Named ranges / regions (generic descriptors)
        "Andean", "Himalayan", "Caucasian", "Apennine",
        "Carpathian", "Pyrenean", "Scandinavian", "Balkan",
        "Anatolian", "Zagros", "Tibetan", "Patagonian",
        "Alaskan", "Cascadian", "Sierra", "Appalachian",
        "Ozark", "Adirondack", "Catskill", "Pocono",
        "Atlas", "Drakensberg", "Ethiopian", "Ruwenzori",
        "Ural", "Altai", "Tian", "Kunlun",
        "Qinling", "Hengduan", "Fuji", "Vesuvian",
        "Etna", "Krakatoan", "Pinatubo", "Merapi",

        // Fault zone / seismic proper nouns (descriptive use)
        "San Andreas", "Hayward", "Cascadia", "Wasatch",
        "New Madrid", "Anatolian", "Zagros", "Sumatra",
        "Nankai", "Tohoku", "Aleutian", "Caribbean",
        "Hellenic", "Calabrian", "Azores", "Canarian",

        // Depth / interior
        "Crustal", "Mantle", "Asthenospheric", "Lithospheric",
        "Subduction", "Obduction", "Accretionary", "Ophiolitic",
        "Moho", "Conrad", "Gutenberg", "Lehmann",
        "Upper-mantle", "Lower-mantle", "Outer-core", "Inner-core",
        "Hypocentric", "Epicentral", "Focal", "Teleseismic",
        "Regional", "Local", "Global", "Microseismic",

        // Network / array descriptors
        "Primary", "Secondary", "Tertiary", "Quaternary",
        "Sentinel", "Guardian", "Watchpost", "Lookout",
        "Relay", "Junction", "Gateway", "Nexus",
        "Anchor", "Keystone", "Pivot", "Apex",
        "Terminal", "Lateral", "Median", "Axial",
        "Baseline", "Reference", "Control", "Calibration",

        // Structural / engineering
        "Bedrock", "Hardrock", "Softrock", "Alluvial",
        "Consolidated", "Loose", "Competent", "Fractured",
        "Weathered", "Fresh", "Altered", "Mineralized",
        "Veined", "Massive", "Banded", "Laminated",
        "Nodular", "Concretionary", "Oolitic", "Bioclastic",

        // Landform specific
        "Moraine", "Drumlin", "Esker", "Kame",
        "Cirque", "Arête", "Horn", "Col",
        "Nunatak", "Fjord", "Loch", "Tarn",
        "Mere", "Fen", "Bog", "Mire",
        "Sinkhole", "Doline", "Polje", "Uvala",
        "Butte", "Mesa", "Hogback", "Cuesta",
        "Inselberg", "Monadnock", "Peneplain", "Pediplain",
        "Alluvial", "Outwash", "Lacustrine", "Aeolian",
        "Loess", "Dune", "Barchan", "Seif",

        // Desert / arid
        "Saharan", "Gobi", "Atacama", "Namib",
        "Arabian", "Patagonian", "Great Basin", "Chihuahuan",
        "Sonoran", "Mojave", "Karakum", "Taklamakan",
        "Negev", "Sinai", "Rub al Khali", "Dasht",

        // Polar / cryosphere
        "Polar", "Circumpolar", "Antarctic", "Arctic",
        "Greenlandic", "Siberian", "Permafrost", "Periglacial",
        "Glaciated", "Deglaciated", "Postglacial", "Ice-capped",
        "Snowbound", "Frost", "Rime", "Hoarfrost",

        // Volcanic
        "Caldera", "Fumarolic", "Solfataric", "Hydrothermal",
        "Lava", "Tephra", "Scoria", "Lapilli",
        "Lahar", "Nuée", "Plinian", "Strombolian",
        "Vulcanian", "Hawaiian", "Fissure", "Shield",
        "Composite", "Cinder", "Maar", "Tuff",

        // Seismological descriptors
        "P-wave", "S-wave", "Surface", "Body",
        "Love", "Rayleigh", "Coda", "Aftershock",
        "Foreshock", "Mainshock", "Tremor", "Swarm",
        "Induced", "Natural", "Tectonic", "Volcanic",
        "Collapse", "Explosion", "Slow", "Silent",
        "Megathrust", "Intraslab", "Interface", "Intraplate",
        "Interplate", "Strike-slip", "Normal", "Reverse",
        "Thrust", "Oblique", "Transpressional", "Transtensional",

        // Additional evocative natural
        "Whispering", "Thundering", "Rumbling", "Trembling",
        "Shaking", "Shifting", "Drifting", "Wandering",
        "Brooding", "Looming", "Watching", "Standing",
        "Sleeping", "Waking", "Rising", "Falling",
        "Broken", "Sunken", "Lifted", "Tilted",
        "Twisted", "Weathered", "Worn", "Carved",
        "Scarred", "Marked", "Hidden", "Exposed",
        "Isolated", "Remote", "Secluded", "Desolate",
        "Barren", "Verdant", "Lush", "Sparse",
        "Rugged", "Jagged", "Smooth", "Polished",
        "Rough", "Craggy", "Sheer", "Gentle",
        "Steep", "Gradual", "Abrupt", "Sudden",
        "Vast", "Endless", "Boundless", "Infinite",
        "Confined", "Enclosed", "Open", "Exposed",
        "Protected", "Sheltered", "Windswept", "Sun-scorched",
        "Frost-bitten", "Rain-soaked", "Mist-shrouded", "Cloud-piercing",
        "Moon-washed", "Star-lit", "Shadow-cast", "Light-bathed",

        // Compass rose variations
        "North", "South", "East", "West",
        "North-central", "South-central", "East-central", "West-central",
        "Far-north", "Far-south", "Far-east", "Far-west",
        "Deep-north", "Deep-south", "Deep-east", "Deep-west",

        // Ordinal descriptors
        "First", "Second", "Third", "Fourth",
        "Alpha", "Beta", "Gamma", "Delta",
        "Prime", "Secondary", "Tertiary", "Auxiliary",

        // Numeric prefixes used as adjectives
        "Mono", "Bi", "Tri", "Quad",
        "Single", "Double", "Triple", "Multiple",

        // Suffix-style qualifiers
        "Long", "Short", "Tall", "Wide",
        "Narrow", "Broad", "Thick", "Thin",
        "Heavy", "Light", "Dense", "Sparse",
        "Fast", "Slow", "Active", "Passive",

        // Extra geological / geographic fillers to reach 1000
        "Subterranean", "Underground", "Subsurface", "Surface",
        "Bedded", "Tilted", "Horizontal", "Vertical",
        "Inclined", "Dipping", "Plunging", "Overturned",
        "Recumbent", "Isoclinal", "Open", "Tight",
        "Closed", "Chevron", "Box", "Ptygmatic",
        "Kink", "Crenulation", "Boudinage", "Mullion",
        "Lineated", "Foliated", "Schistose", "Gneissic",
        "Mylonitic", "Cataclastic", "Brecciated", "Sheared",
        "Augen", "Porphyroblastic", "Porphyritic", "Aphanitic",
        "Pegmatitic", "Aplitic", "Lamprophyric", "Kimberlitic",
        "Carbonatitic", "Syenitic", "Dioritic", "Gabbroic",
        "Peridotitic", "Dunitic", "Pyroxenitic", "Harzburgitic",
        "Lherzolitic", "Wehrlitic", "Websteritic", "Eclogitic",
        "Blueschist", "Greenschist", "Amphibolitic", "Granulitic",
        "Migmatitic", "Anatexite", "Restitic", "Xenolithic",
        "Enclaved", "Cumulate", "Layered", "Banded",
        "Orbicular", "Spherulitic", "Variolitic", "Amygdaloidal",
        "Vesicular", "Scoriaceous", "Tuffaceous", "Agglomeratic",

        // Hydrological / fluvial detail
        "Braided", "Meandering", "Anastomosing", "Entrenched",
        "Incised", "Aggrading", "Degrading", "Perched",
        "Ephemeral", "Intermittent", "Perennial", "Gaining",
        "Losing", "Influent", "Effluent", "Confluent",
        "Distributary", "Tributary", "Headwater", "Thalweg",
        "Oxbow", "Meander", "Scroll", "Cutbank",

        // Pedological / soil
        "Lateritic", "Ferruginous", "Calcareous", "Gypsic",
        "Salic", "Natric", "Argillic", "Spodic",
        "Oxic", "Kandic", "Petrocalcic", "Petrogypsic",
        "Duripan", "Fragipan", "Claypan", "Hardpan",
        "Plinthic", "Vertic", "Andic", "Histic",
        "Mollic", "Umbric", "Ochric", "Albic",

        // Permafrost / periglacial detail
        "Patterned", "Polygonal", "Striped", "Sorted",
        "Unsorted", "Active-layer", "Thermokarst", "Talik",
        "Pingo", "Palsa", "Yedoma", "Aufeis",
        "Solifluction", "Gelifluction", "Cryoturbated", "Ice-wedge",

        // Structural geology detail
        "En-echelon", "Sigmoidal", "Anastomosing", "Flower",
        "Pop-up", "Wedge", "Duplex", "Imbricate",
        "Ramp", "Flat", "Blind", "Emergent",
        "Listric", "Planar", "Curved", "Branching",
        "Detachment", "Décollement", "Basal", "Roof",

        // Remote / landscape character
        "Trackless", "Pathless", "Roadless", "Unmarked",
        "Uncharted", "Unexplored", "Pristine", "Untouched",
        "Primeval", "Primordial", "Timeless", "Ageless",
        "Forgotten", "Lost", "Hidden", "Secret",
        "Silent", "Hushed", "Muted", "Resonant",
        "Echoing", "Ringing", "Humming", "Pulsing",
        "Vibrating", "Oscillating", "Rhythmic", "Periodic",

        // Observatory / scientific flavour
        "Monitoring", "Recording", "Sensing", "Detecting",
        "Measuring", "Sampling", "Logging", "Reporting",
        "Transmitting", "Receiving", "Processing", "Archiving",
        "Broadband", "Short-period", "Long-period", "Strong-motion",
        "Accelerometric", "Velocimetric", "Gravimetric", "Geodetic",
        "Infrasound", "Hydroacoustic", "Ionospheric", "Atmospheric",

        // Additional evocative / poetic
        "Stoic", "Vigilant", "Steadfast", "Unwavering",
        "Resilient", "Enduring", "Patient", "Persistent",
        "Resolute", "Stalwart", "Steadfast", "Unyielding",
        "Timeworn", "Battered", "Tested", "Proven",
        "Solitary", "Lone", "Singular", "Unique",
        "Stark", "Spare", "Austere", "Minimal",
        "Profound", "Immense", "Intimate", "Peripheral",
        "Marginal", "Liminal", "Threshold", "Transitional",

        // Topographic and atmospheric
        "Windswept", "Storm-beaten", "Rain-carved", "Frost-shattered",
        "Sun-baked", "Ice-polished", "Wave-cut", "Wind-eroded",
        "Tsunami", "Seiche", "Surge", "Inundated",
        "Upwelling", "Downwelling", "Convective", "Advective",
        "Radiative", "Conductive", "Diffusive", "Dispersive",
        "Reflective", "Refractive", "Diffractive", "Absorptive",
        "Attenuating", "Amplifying", "Resonant", "Dampened",
        "Stiff", "Soft", "Hard", "Yielding",
        "Locked", "Creeping", "Coupled", "Decoupled",
        "Loaded", "Unloaded", "Stressed", "Relaxed",
        "Nucleating", "Propagating", "Arresting", "Healing",
    ]
}
