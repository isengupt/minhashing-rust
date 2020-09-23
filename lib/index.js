var addon = require('../native');
const fetch = require('node-fetch');
//ghJXTwmIQ58zDPPHrANkaHDiDQc8jJgn
//n6Js7p0QJxyv6uP2


fetch('https://api.nytimes.com/svc/archive/v1/2019/1.json?api-key=ghJXTwmIQ58zDPPHrANkaHDiDQc8jJgn')
    .then(res => res.json())
    .then(json => {
       // console.log(json.response.docs)
 
    const newJson =  json.response.docs.map((item) => {
        if (!(item.lead_paragraph === "undefined")) {
         
            return {_id: item._id, text: item.lead_paragraph}
        }
     })
       //console.log(newJson)
       addon.debugArrayOfObjects(newJson.slice(1,100))
    }
    );