# AgentToolAccessRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**agent_protocol** | **bool** |  | [optional] 
**coding_agent_job** | **bool** |  | [optional] 
**command** | **bool** |  | [optional] 
**filesystem** | **bool** |  | [optional] 
**sub_agent** | **bool** | Enable the &#x60;execute_sub_agent&#x60; tool for spawning child agents. | [optional] 
**url_fetch** | **bool** | Enable the &#x60;fetch_url&#x60; URL scraping tool (Firecrawl API). | [optional] 
**web_search** | **bool** | Enable the &#x60;search_with_llm&#x60; web search tool (Google Custom Search). | [optional] 

## Example

```python
from tachyon_sdk.models.agent_tool_access_request import AgentToolAccessRequest

# TODO update the JSON string below
json = "{}"
# create an instance of AgentToolAccessRequest from a JSON string
agent_tool_access_request_instance = AgentToolAccessRequest.from_json(json)
# print the JSON string representation of the object
print(AgentToolAccessRequest.to_json())

# convert the object into a dict
agent_tool_access_request_dict = agent_tool_access_request_instance.to_dict()
# create an instance of AgentToolAccessRequest from a dict
agent_tool_access_request_from_dict = AgentToolAccessRequest.from_dict(agent_tool_access_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


