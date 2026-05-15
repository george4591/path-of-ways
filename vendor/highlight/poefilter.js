/*
Language: PoE Filter
Description: Path of Exile / Path of Exile 2 loot filter syntax
Aliases: filter, poe, poe2
*/
(function () {
    function poefilter(hljs) {
        var KEYWORDS = {
            keyword: 'Show Hide Continue Minimal Import',
            built_in:
                'Class BaseType Rarity ItemLevel DropLevel AreaLevel MapTier WaystoneTier ' +
                'Quality Sockets LinkedSockets SocketGroup Width Height StackSize ' +
                'GemLevel GemQualityType AlternateQuality TransfiguredGem ' +
                'Identified Corrupted Mirrored ElderItem ShaperItem FracturedItem SynthesisedItem ' +
                'AnyEnchantment HasEnchantment EnchantmentPassiveNum EnchantmentPassiveNode ' +
                'HasInfluence HasExplicitMod HasImplicitMod ' +
                'HasEaterOfWorldsImplicit HasSearingExarchImplicit ' +
                'BlightedMap UberBlightedMap Scourged ArchnemesisMod Replica',
            'title.function':
                'SetTextColor SetBackgroundColor SetBorderColor SetFontSize ' +
                'PlayAlertSound PlayAlertSoundPositional ' +
                'CustomAlertSound CustomAlertSoundOptional ' +
                'MinimapIcon PlayEffect ' +
                'DisableDropSound EnableDropSound ' +
                'DisableDropSoundIfAlertSound EnableDropSoundIfAlertSound',
            literal:
                'True False ' +
                'Normal Magic Rare Unique ' +
                'Red Green Blue Brown White Yellow Cyan Grey Orange Pink Purple Temp ' +
                'Circle Diamond Hexagon Square Star Triangle UpsideDownHouse Cross Moon Raindrop Kite Pentagon ' +
                'Shaper Elder Crusader Hunter Redeemer Warlord'
        };

        return {
            name: 'PoE Filter',
            aliases: ['filter', 'poe', 'poe2'],
            case_insensitive: false,
            keywords: KEYWORDS,
            contains: [
                hljs.HASH_COMMENT_MODE,
                hljs.QUOTE_STRING_MODE,
                hljs.NUMBER_MODE,
                {
                    className: 'operator',
                    match: /==|>=|<=|!=|[<>=!]/
                }
            ]
        };
    }

    if (typeof hljs !== 'undefined') {
        hljs.registerLanguage('poefilter', poefilter);
    }
})();
