# default_api

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
****](default_api.md#) | **GET** /cats | Get all cats
****](default_api.md#) | **DELETE** /cats/{id} | Delete a cat by ID
****](default_api.md#) | **GET** /cats/{id} | Get a cat by ID
****](default_api.md#) | **PUT** /cats/{id} | Update a cat by ID
****](default_api.md#) | **POST** /cats | Create a new cat
****](default_api.md#) | **GET** /dogs | Get all dogs
****](default_api.md#) | **DELETE** /dogs/{id} | Delete a dog by ID
****](default_api.md#) | **GET** /dogs/{id} | Get a dog by ID
****](default_api.md#) | **PUT** /dogs/{id} | Update a dog by ID
****](default_api.md#) | **POST** /dogs | Create a new dog


# ****
> Vec<models::Cat> ()
Get all cats

### Required Parameters
This endpoint does not need any parameter.

### Return type

[**Vec<models::Cat>**](Cat.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# ****
> (id)
Delete a cat by ID

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **id** | **String**|  | 

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# ****
> models::Cat (id)
Get a cat by ID

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **id** | **String**|  | 

### Return type

[**models::Cat**](Cat.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# ****
> models::Cat (id, cat)
Update a cat by ID

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **id** | **String**|  | 
  **cat** | [**Cat**](Cat.md)|  | 

### Return type

[**models::Cat**](Cat.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# ****
> models::Cat (cat)
Create a new cat

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **cat** | [**Cat**](Cat.md)|  | 

### Return type

[**models::Cat**](Cat.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# ****
> Vec<models::Dog> ()
Get all dogs

### Required Parameters
This endpoint does not need any parameter.

### Return type

[**Vec<models::Dog>**](Dog.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# ****
> (id)
Delete a dog by ID

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **id** | **String**|  | 

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# ****
> models::Dog (id)
Get a dog by ID

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **id** | **String**|  | 

### Return type

[**models::Dog**](Dog.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# ****
> models::Dog (id, dog)
Update a dog by ID

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **id** | **String**|  | 
  **dog** | [**Dog**](Dog.md)|  | 

### Return type

[**models::Dog**](Dog.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# ****
> models::Dog (dog)
Create a new dog

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **dog** | [**Dog**](Dog.md)|  | 

### Return type

[**models::Dog**](Dog.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

