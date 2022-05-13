% GIROUETTE(1) Version 0.7.2 | Girouette Usage Documentation

NAME
====

**girouette** — displays the weather

SYNOPSIS
========

| **girouette** \[_OPTIONS_]... \[**-l**|**\--location** _location_]
| **girouette** \[**\--clean-cache**|**\--print-default-config**]
| **girouette** \[**-h**|**\--help**|**-V**|**\--version**]

DESCRIPTION
===========

Display the current weather using the Openweather API.

OPTIONS
=======

Query options
-------------

-c, \--cache _DURATION_

:   Cache responses for this long (e.g. _1m_, _2 days 6h_, _5 sec_), or _none_ to disable it.

    If there is a cached response younger than the duration given as argument, it  is returned directly. Otherwise, it queries the API and write the response to the cache for use by a later invocation.

    NOTE: No response is written to the cache if this option isn't set. The invocation doing the caching and the one potentially querying it both need this option set.

    Recognized durations go from seconds (_seconds_, _second_, _sec_, _s_) to years (_years_, _year_, _y_).

    This option overrides the corresponding value from the config.

\--config _FILE_

:   Use the specified configuration file instead of the default.

    By default, girouette looks for a configuration file:
            
    - on Linux in _`$XDG_CONFIG_HOME/girouette/config.yml`_ or _`$HOME/.config/girouette/config.yml`_
    - on MacOS in _`$HOME/Library/Application Support/rs.Girouette/config.yml`_
    - on Windows in _`%AppData%\Girouette\config\config.yml`_

-k, \--key _KEY_

:   OpenWeather API key (required for anything more than light testing).

    Can be either the key or an '_`@`_' character followed by a path to a file containing the key. The path can either: 

    - be a relative path:  will be resolved relative to girouette's config directory
    - start with '_~/_': will be resolved relative to the user's home directory
    - be an absolute path.

    This option overrides the corresponding value from the config.

-l, \--location _LOCATION_

:   Location to query (required if not set in config).

    Possible values are:

    - Location names: '_`London, UK`_', '_`Dubai`_'
    - Geographic coordinates (lat,lon): '_`35.68,139.69`_'

    This option overrides the corresponding value from the config.

 -L, \--language _LANGUAGE_

:   Use this language for location names, weather descriptions and date formatting.

    This asks OpenWeather to provide location names and weather descriptions in the given language, and uses it to format date and times.

    Possible values are of the form _`aa_AA`_ like _`en_US`_ or _`fr_FR`_. Note that OpenWeather only supports a subset of all valid **LANG** values.

    This option overrides the corresponding value from the config.

-u, \--units _UNIT_

:   Units to use when displaying temperatures and speeds.

    Possible units are:

    - _metric_: Celsius temperatures and kilometers/hour speeds (the default),
    - _imperial_: Fahrenheit temperatures and miles/hour speeds,
    - _standard_: Kelvin temperatures and meters/second speeds.

    This option overrides the corresponding value from the config.

Flags
-----

-o, \--offline

:   Run only offline with responses from the cache.

    The cache is used unconditionally, regardless of the cache length given in the configuration file. The network is never queried.

    If there is no cached response for this particular location, an error will be returned.

-q, \--quiet

:   Pass for less log output

-v, \--verbose

:   Pass for more log output

Global commands
---------------

\--clean-cache
:   Removes all cached responses and exits.

    This empties the cache directory used when caching responses with **`-c/--cache`**.

    By default, girouette puts the cache in:

    - on Linux in _`$XDG_CACHE_HOME/girouette/results/`_ or _`$HOME/.cache/girouette/results/`_
    - on MacOS in _`$HOME/Library/Caches/rs.Girouette/results/`_
    - on Windows in _`%AppData%\Girouette\cache\results\`_

\--print-default-config
:   Prints the contents of the default configuration and exits.

    This allows creating a new configuration using the default configuration as a template.

Info
----
-h, \--help

:   Print help information

-V, \--version

:   Print version information

FILES
=====

On Linux
--------

_\$XDG_CONFIG_HOME/girouette/config.yml_ or _~/.config/girouette/config.yml_

:   Default configuration file.

_\$XDG_CACHE_HOME/girouette/results/_ or _~/.cache/girouette/results/_

:   Default cache directory.

On MacOS
--------

_~/Library/Application Support/rs.Girouette/config.yml_

:   Default configuration file.

_~/Library/Caches/rs.Girouette/results/_

:   Default cache directory.

On Windows
----------

_%AppData%\\Girouette\\config\\config.yml_

:   Default configuration file.

_%AppData%\\Girouette\\cache\\results\\_

:   Default cache directory.

ENVIRONMENT
===========

**LANG**

:   The default display language if none is given.

BUGS
====

See GitHub Issues: <https://github.com/gourlaysama/girouette/issues>

AUTHOR
======

Antoine Gourlay <antoine@gourlay.fr>
